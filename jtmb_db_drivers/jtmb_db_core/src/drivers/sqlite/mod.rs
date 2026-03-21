use std::{
    clone, io::{BufRead, Read}, ops::DerefMut, random::random, sync::{Arc, atomic::AtomicU32, mpsc::TryRecvError}, thread::JoinHandle, time::Duration, u32
};
pub mod utils;
pub mod types;
pub mod vtbl;
pub mod worker;


unsafe extern "C" {
    /// Manually linking the v2 close function
    pub fn sqlite3_close_v2(db: *mut sqlite3) -> std::ffi::c_int;
}

use crossbeam::{queue::ArrayQueue};
use dashmap::DashMap;
use libsqlite3_sys::*;

use crate::drivers::sqlite::worker::{WorkerResponse, WorkerTask};
type WorkerId = u32;
pub struct Driver {
    options: DriverOptions,
    workers: ArrayQueue<self::worker::Worker>,
    connect_options: ConnectOptions,
}
#[unsafe(no_mangle)]
unsafe extern "C" fn sqlite_log_callback(
    _user_data: *mut std::ffi::c_void,
    err_code: i32,
    msg: *const std::ffi::c_char,
) {
    let msg = unsafe { std::ffi::CStr::from_ptr(msg) }.to_string_lossy();
    eprintln!("[SQLite Log] ({}) {}", err_code, msg);
}

impl Driver {
    fn sqlite3_initialize() -> std::io::Result<()> {
        match unsafe { sqlite3_initialize() } {
            SQLITE_OK => Ok(()),
            SQLITE_ERR => todo!(),
            SQLITE_MISUSE => todo!("misuse"),
            _ => todo!("Unknown"),
        }
    }
    fn sqlite3_is_threadsafe() -> i32 {
        unsafe { sqlite3_threadsafe() }
    }
    fn sqlite3_config_log() -> i32 {
        unsafe {
            sqlite3_config(
                SQLITE_CONFIG_LOG,
                std::ptr::from_ref(&sqlite_log_callback),
                std::ptr::null_mut::<std::ffi::c_void>(),
            )
        }
    }
    fn sqlite3_config_uri(allow: bool) -> std::io::Result<()> {
        match unsafe { sqlite3_config(SQLITE_CONFIG_URI, allow as i32) } {
            SQLITE_OK => Ok(()),
            SQLITE_MISUSE => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Attempted to configure SQLITE_CONFIG_SERIALIZED after initializing sqlite. make sure you call shutdown() first",
            )),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unknown Sqlite Error while attempting to call sqlite3_cosqlite3_config_uri",
            )),
        }
    }
    fn sqlite3_config_serialized(mode: i32) -> std::io::Result<()> {
        match unsafe { sqlite3_config(SQLITE_CONFIG_SERIALIZED, mode) } {
            SQLITE_OK => Ok(()),
            SQLITE_ERROR => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "SQlite Error: Attempted to configure SQLITE_CONFIG_SERIALIZED, But The linked library was compiled without thread-safety (SQLITE_THREADSAFE=0).",
                ));
            }
            SQLITE_MISUSE => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Attempted to configure SQLITE_CONFIG_SERIALIZED after initializing sqlite. make sure you call shutdown() first",
                ));
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unknown Sqlite Error while attempting to call sqlite3_config_serialized",
            )),
        }
    }
}

#[repr(transparent)]
/// do not SYNC conns between threads, do not SHARE conns between workers only SEND them to the worker thread;
/// it is UNSAFE to attempt to share this type between threads. as such it does NOT and WILL NOT implement clone or copy.
/// if you abuse the deref and deref mut implementation to do something nasty, Author CANNOT GUARENTEE that it will be safe.
/// unsafe impl SEND only allows it to be SENT to the worker thread it was meant for
pub(crate) struct ConnectionHandle(*mut sqlite3);
impl std::ops::Deref for ConnectionHandle {
    type Target = *mut sqlite3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ConnectionHandle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for ConnectionHandle {}
// unsafe impl !Sync for ConnectionHandle{}

// WARNING: DROP MUST BE CALLED BEFORE WORKER is dropped.
// as long as connectionhandle goes out of scope when the worker exits
// you should be fine

impl Drop for ConnectionHandle {
    fn drop(&mut self) {
        // connection handle should never be null on drop
        debug_assert!(self.0.is_null() == false);
        unsafe {
            // attempt to call interrupt to get sqlite3 to stop whatever its doing. our connection is geting closed
            sqlite3_interrupt(self.0);

            let r = sqlite3_close_v2(self.0);
            // let r = sqlite3_close(self.0);
            if r != SQLITE_OK {
                eprintln!(
                    "Call to sqlite3_close failed on implicit drop for ConnectionHandle: sqlite3_close returned: {r}"
                )
            }
        }
    }
}

pub struct DriverOptions {
    thread_mode: i32,
    allow_uris: bool,
    num_worker_threads: u32,
    // connect_options: ConnectOptions,
}
#[derive(Clone, Debug)]
pub struct ConnectOptions {
    url: String,
}

impl crate::DriverInitialize for self::Driver {
    type InitOptions = self::DriverOptions;
    type ConnectionOptions = self::ConnectOptions;
    fn driver_initialize(
        driver_options: Self::InitOptions,
        connect_options: Self::ConnectionOptions,
    ) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        // configure sqlite before we start
        // if is_threadsafe > 0 then call config serialized
        if Self::sqlite3_is_threadsafe() > 0 {
            Self::sqlite3_config_serialized(driver_options.thread_mode)?;
        }
        Self::sqlite3_config_uri(driver_options.allow_uris)?;
        let _ = Self::sqlite3_config_log();

        Self::sqlite3_initialize()?;

        // NOTE if you try to spawn more than 2^32 workers, the worker id  will wrap around
        // and drop the 0'th worker, but if you do that you have bigger problems
        let mut worker_id = 0_u32;
        let mut workers = ArrayQueue::new(driver_options.num_worker_threads as usize);
        for _ in 0..driver_options.num_worker_threads {
            let worker = self::worker::Worker::spawn(&connect_options)?;
            unsafe { worker_id = worker_id.unchecked_add(1) };
            workers.push(worker).unwrap();
        }

        Ok(Self {
            options: driver_options,
            connect_options,
            workers: workers,
        })
    }
}

// this drop impl uses while !is_empty and pop to force worker's drop implementation
// to be called when drop is called

pub fn calculate_logarythmic_backoff(retry_count: u32) -> Duration {
    let retry_slope = 10.0f64;
    let max_retry_count = 655350.0;
    let min_duration_micros = 10.0;
    let max_duration_micros = 1000.0;
    let retry_count = f64::min(max_retry_count, retry_count as f64);
    let slope = (min_duration_micros - min_duration_micros) / f64::ln(max_retry_count);
    let micros = min_duration_micros + (slope * f64::ln(retry_slope));
    // std::random::DefaultRandomSource::
    let jitter: u8 = random(..);
    let total_wait_micros = (micros as u64) + jitter as u64;
    let duration = std::time::Duration::from_micros(total_wait_micros);
    duration
}

impl Drop for Driver {
    fn drop(&mut self) {
        // drop(self.workers);
        while !self.workers.is_empty() {
            let _ = self.workers.pop();
        }
        // shutdown the driver
        unsafe {
            let r = sqlite3_shutdown();
            if r != SQLITE_OK {
                eprintln!("sqlite3_shutdown call failed  {r}")
            }
        }
    }
}

impl Driver {
    // pub fn query(&mut self, q: &str) -> std::io::Result<()>{
    //     let mut w = self.workers.pop();
    //     let retry= 0;
    //     while retry < 65535 {
    //         // no workers available
    //         w = self.workers.pop();
    //         if w.is_none() {
    //             let wait_time = calculate_logarythmic_backoff(retry);
    //             std::thread::park_timeout(wait_time);
    //         }
    //         if retry > 65535 {
    //             return Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "Failed to aquire a worker in 65535 * 10 microseconds. try again"));
    //         }
    //         let worker = w.take().unwrap();
    //         if worker.status.load(std::sync::atomic::Ordering::Acquire) != 0 {
    //             self.workers.push(worker).unwrap();
    //         }
    //         else {
    //             println!("a");
    //             worker.thread_handle.as_ref().unwrap().thread().unpark();
    //             let (tx,rx) = std::sync::oneshot::channel();
    //             let _ = worker.sender.send(WorkerTask::Execute(q.to_string(),tx));
    //             println!("waiting...");

    //             let msg = rx.recv().unwrap();
    //             println!("a");

    //             println!("thread executed query");
    //             self.workers.push(worker).unwrap();
    //             return Ok(())
    //         }
    //     }
    //     Ok(())
    // }

    pub fn query(&mut self, q: &str) -> std::io::Result<std::sync::mpsc::Receiver<WorkerResponse>> {
        let mut retry = 0;

        // 1. Loop until we successfully POP a worker or hit the timeout
        let worker = loop {
            if let Some(w) = self.workers.pop() {
                break w; // We got a worker!
            }

            retry += 1;
            if retry >= 65535 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::WouldBlock,
                    "Driver Timeout: No workers available in the pool",
                ));
            }

            // Logarithmic backoff while waiting for a worker to be pushed back
            let wait_time = calculate_logarythmic_backoff(retry);
            std::thread::park_timeout(wait_time);
        };

        // 2. Setup the "Consistency Handshake"
        let (tx, rx) = std::sync::mpsc::channel();

        // 3. Dispatch the task
        // We unpark before sending to minimize the gap between "task in queue" and "worker awake"
        // worker.thread_handle.as_ref().unwrap().thread().unpark();

        if let Err(_) = worker.send(WorkerTask::Execute(q.to_string(), tx)) {
            // If the worker thread died, we still need to push it back or handle the loss
            self.workers.push(worker).ok();
            return Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Worker thread died",
            ));
        }

        // 4. BLOCK: Wait for SQLite to finish the work
        // The oneshot::Receiver::recv() is the ultimate consistency guarantee
        // let _rc = rx.recv().map_err(|_| {
        //     std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         "Worker dropped responder (likely panic)",
        //     )
        // })?;

        // 5. RELEASE: Return the worker to the back of the queue for the next caller
        self.workers.push(worker).unwrap();

        Ok(rx)
    }
}

#[test]
pub fn test_reactor() -> std::io::Result<()> {
    use crate::DriverInitialize;
    // stress test, call this 20 times in a row. should not panic or error

    for _ in 0..1 {
        let connect_options = ConnectOptions {
            url: ":memory:".to_string(),
        };
        let driver_options = DriverOptions {
            allow_uris: true,
            thread_mode: SQLITE_CONFIG_SINGLETHREAD,
            num_worker_threads: 1,
        };
        let mut driver = Driver::driver_initialize(driver_options, connect_options)?;
        for _ in 0..1 {
            let r = driver.query(r#"SELECT 
    42                 AS id,             -- SQLITE_INTEGER (1)
    3.14159            AS pi_value,       -- SQLITE_FLOAT   (2)
    'Reactor Core'     AS driver_name,    -- SQLITE_TEXT    (3)
    NULL               AS empty_slot,     -- SQLITE_NULL    (5)
    CAST('X' AS BLOB)  AS raw_data        -- SQLITE_BLOB    (4)"#)?;
            let s = std::thread::spawn(move || {
                let mut schema = Vec::new();
                while let Ok(message) = r.recv() {
                    let row = match message {
                        WorkerResponse::Schema(s) => {schema = s; continue;},
                        WorkerResponse::PrepareFailed(e) => {
                            eprintln!("Query Plan failed: {e}");
                            break;
                        },
                        WorkerResponse::Done => {
                            eprintln!("Done");
                            break;
                        }
                        WorkerResponse::Row(row)=>{
                            dbg!(row.len());
                            row
                            
                        }
                    };
                    for b in &row {
                        print!("{b:08b} ");
                    }
                    println!();
                    // 1st val col_type as i32
                    let len = row.len();
                    let mut cursor = std::io::Cursor::new(row);
                    while cursor.position() < len as u64 {

                        let mut integer_bytes = [0_u8;std::mem::size_of::<i32>()];
                        cursor.read_exact(&mut integer_bytes).unwrap();
                        
                        let type_id = i32::from_ne_bytes(integer_bytes);
                        dbg!(type_id);
                        let mut integer_bytes = [0_u8;std::mem::size_of::<u32>()];
                        cursor.read_exact(&mut integer_bytes).unwrap();
                        
                        let len = u32::from_ne_bytes(integer_bytes);
                        dbg!(len);
                        let len2 = len as usize;
                        let mut data_bytes = Vec::<u8>::with_capacity(10);
                        cursor.read_exact(&mut data_bytes).unwrap();
                        
                        match type_id {
                            SQLITE_INTEGER => {
                                let i = i64::from_ne_bytes(data_bytes.try_into().unwrap());
                                dbg!(i);
                            }
                            SQLITE_FLOAT => {
                                let f = f64::from_ne_bytes(data_bytes[0_usize..len as usize].try_into().unwrap());
                                dbg!(f);
                            }
                            SQLITE3_TEXT => {
                                let s = String::from_utf8(data_bytes).unwrap();
                                dbg!(s);
                            }
                            SQLITE_BLOB => {
                                dbg!("BLOB");
                            }
                            SQLITE_NULL => {
                                dbg!("NULL COLUMN");
                            }
                            _=>{
                                dbg!(type_id);
                            }
                        }
                    }

                    // dbg!(type_id,len,data_bytes);
                    

                }
                // let r = r.recv();



                println!("sql thread sent {r:?}")
            });
            s.join();
        }
    }

    // std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(())
}
