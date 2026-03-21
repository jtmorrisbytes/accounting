use std::{sync::Arc, thread::JoinHandle};

use libsqlite3_sys::*;

use crate::drivers::sqlite::{ConnectOptions, ConnectionHandle};


#[derive(Debug)]
pub enum WorkerResponse {
    PrepareFailed(String),
    Schema(Vec<(String, i32)>),
    Row(Vec<u8>),
    Done,
}



pub enum WorkerTask {
    Execute(String, std::sync::mpsc::Sender<WorkerResponse>),
    Shutdown,
}

/// DROP must be called after the connection exits but before the driver gets dropped

impl Drop for Worker {
    fn drop(&mut self) {
        // attempt to send the message to the thread
        let handle = self
            .thread_handle
            .take()
            .expect("BUG BUG! Worker thread should be initialized");

        handle.thread().unpark();

        match self.sender.send(WorkerTask::Shutdown) {
            Err(e) => {
                println!(
                    "WARNING: failed to send shutdown signal to worker thread: {e}. connection may dangle."
                );
            }
            _ => {}
        }
        let r = handle.join();
        println!("thread joined with result {r:?}");
    }
}



pub(crate) unsafe fn worker_execute(stmt: &mut super::types::Statement,sender:std::sync::mpsc::Sender<WorkerResponse>,row_data: &mut Vec<u8>) -> std::io::Result<()> {
    let mut rc = stmt.step().unwrap();
    // let num_columns = sqlite3_column_count(**stmt);
    let num_columns = stmt.column_count();
    // let mut rc = sqlite3_step(stmt);

    let mut metadata = Vec::new();
    for i in 0..num_columns {
        let col_type = unsafe { sqlite3_column_type(**stmt, i) };
        dbg!(col_type);
        let n = unsafe {
            let name_ptr = unsafe { sqlite3_column_name(**stmt, i as i32) };
            if name_ptr.is_null() {
                metadata.push((String::new(), col_type));
            }
            std::ffi::CStr::from_ptr(name_ptr)
                .to_string_lossy()
                .to_string()
        };
        metadata.push((n, col_type))
    }
    sender.send(WorkerResponse::Schema(metadata.clone())).ok();
    let mut row_data = Vec::<u8>::with_capacity(256);
    while rc != SQLITE_DONE {
        for c in 0..num_columns {
            // fast vtble insead of column match
            unsafe {
                super::vtbl::EXTRACT_TBL[c as usize](**stmt, c as i32, &mut row_data)
            }
        }
        rc = stmt.step().unwrap();
        println!("sqlite3_step {rc}");
    }
    if rc != SQLITE_DONE {
        println!("Sqlite3 step returned a non DONE response {rc}");
    }
    stmt.finalize();
    // let r = sqlite3_finalize(**stmt);
    // println!("sqlite3_finalize {r}");
    sender.send(WorkerResponse::Done).ok();
    Ok(())
}



#[derive(Debug)]
pub struct Worker {
    // connection: *mut sqlite3,
    connection_options: ConnectOptions,
    // status: Arc<AtomicU32>,
    sender: std::sync::mpsc::Sender<WorkerTask>,
    thread_handle: Option<std::thread::JoinHandle<Result<(), std::io::Error>>>,
}
impl Worker {
    pub(crate) fn send(&self,task: WorkerTask) -> Result<(), std::sync::mpsc::SendError<WorkerTask>> {
        self.sender.send(task)
    }


    pub(crate) fn spawn(options: &ConnectOptions) -> std::io::Result<Self> {
        let url = std::ffi::CString::new(options.url.as_str())?;
        let (task_sender, task_reviever) = std::sync::mpsc::channel();
        let mut conn_ptr: *mut sqlite3 = std::ptr::null_mut();
        let result = unsafe {
            sqlite3_open_v2(
                url.as_ptr(),
                &mut conn_ptr,
                SQLITE_OPEN_READWRITE | SQLITE_OPEN_CREATE,
                core::ptr::null(),
            )
        };
        if result != SQLITE_OK {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                format!(
                    "Failed to connect to sqlite DB. sqlite returned error code {result} in call to sqlite3_v2_open"
                ),
            ));
        }
        let conn = conn_ptr;
        let conn = ConnectionHandle(conn);
        // let status = Arc::new(AtomicU32::new(u32::MAX));

        // let status1 = status.clone();
        let thread_handle: JoinHandle<std::io::Result<()>> = std::thread::spawn(move || {
            let conn = conn;
            // let status = status1;
            println!("Worker thread spawned");
            loop {
                let m = match task_reviever.recv() {
                    Err(e) => {
                        // sender disconnected. thread may be dropped
                        println!("Warning: Worker thread detected that task sender disconnected");
                        // todo!();
                        break;
                        // Driver::try_disconnect_on_worker_sender_dropped(conn_ptr);
                    }
                    // if the queue is empty, the thread will park
                    Ok(m) => m,

                };
                match m {
                    WorkerTask::Execute(sql, sender) => {
                        let mut stmt = super::types::Statement::prepare(&conn, &sql).unwrap();
                        let mut row_data = Vec::with_capacity(128);
                        unsafe {worker_execute(&mut stmt, sender, &mut row_data)};
                        row_data.clear();
                    }
                    WorkerTask::Shutdown => {
                        println!("Shutdown requested");
                        break;
                    }
                }
            }
            std::io::Result::Ok(())
        });
        let s = Self {
            connection_options: options.to_owned(),
            sender: task_sender,
            thread_handle: Some(thread_handle),
            // status: status,
        };
        Ok(s)
    }
}
