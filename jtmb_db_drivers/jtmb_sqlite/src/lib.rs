// any core ffi functions that can frequently change or need grouping go here
// otherwise put the ffi interfaces in the place where they are used with feature gates

pub mod alloc;
pub mod config;
pub mod ffi;

pub type Error = Box<dyn std::error::Error>;


/// core connection type
/// connections not publicly available in this api
// WARNING: DO NOT SEND or share BETWEEN THREADS>.. EXPERIMENTAL
#[derive(Debug)]
pub(crate) struct SqliteContext {
    database: *mut self::ffi::sqlite3,
    stmt: *mut self::ffi::sqlite3_stmt,
    _stmt_buffer: Vec<u8>
}

impl SqliteContext {
    pub fn from_raw(handle: *mut self::ffi::sqlite3,stmt: *mut self::ffi::sqlite3_stmt) -> Self {
        Self {
            database: handle,
            stmt,
            _stmt_buffer: Vec::new()
        }
    }
    pub fn reset_stmt(&mut self) -> i32{
        debug_assert!(self.database.is_null() == false);
        debug_assert!(self.stmt.is_null() == false);
        unsafe {
            self::ffi::sqlite3_reset(self.stmt)
        }
    }
    pub fn from_initialized_without_prepared_statement(handle: *mut self::ffi::sqlite3) -> Self {
        // if handle is null, we programmed it wrong. only checked on non release builds
        debug_assert!(handle.is_null() == false);
        Self {
            database: handle,
            stmt: core::ptr::null_mut(),
            _stmt_buffer: Vec::new()
        }
    }
    // prepares the sql, overwriting any previous statement
    // this should be called after opening the database
    pub fn prepare_sql<S: AsRef<str>>(&mut self,s:S) -> i32 {
    debug_assert!(self.database.is_null() == false);

    let sql = s.as_ref();
    // let mut stmt = ptr::null_mut();
    // collect the bytes into an owned buffer for the ffi call. 
    let sql: Vec<u8> = sql.bytes().chain(Some(0)).collect();
    self._stmt_buffer = sql;
    let res = unsafe {crate::ffi::sqlite3_prepare_v2(self.database, self._stmt_buffer.as_ptr().cast(), self._stmt_buffer.len().try_into().unwrap(), &mut self.stmt, ptr::null_mut())};
    if res != self::ffi::SQLITE_OK as i32 { return -1; }
        return 0
    }
    pub fn finalize_statement(&mut self) -> i32 {
        debug_assert!(self.stmt.is_null() == false);
        unsafe {
            // finalize invalidates the pointer
            let r = self::ffi::sqlite3_finalize(self.stmt);
            self.stmt = std::ptr::null_mut();
            return r;
        }
    }
}


impl Drop for SqliteContext {
    fn drop(&mut self) {
        unsafe {
            if !self.stmt.is_null() {
                crate::ffi::sqlite3_finalize(self.stmt);
            }
            if !self.database.is_null() {
                crate::ffi::sqlite3_close_v2(self.database);
            }
        }
    }
}


// the only 'hack' we will probably use. we are guarenteing the compiler that we wont move this between threads
unsafe impl Send for SqliteContext{}
// unsafe impl Sync for Connection{}



pub fn sqlite3_init(parameters: u32) -> Result<(), self::Error> {
    unsafe {
        config::jtmb_sqlite_configure(parameters)?;
        let r = ffi::sqlite3_initialize();
        match r.try_into()? {
            // ffi::SQliteZigInitResult::Ok => Ok(()),
            ffi::SQLITE_OK => {
                return Ok(())
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Unknown initialization error",
                )));
            }
            _ => {}
        }
        Ok(())
    }
}

pub fn sqlite3_shutdown() -> Result<(), self::Error> {
    unsafe {
        let r = ffi::sqlite3_shutdown();
        match r {
            ok if ffi::SQLITE_OK as i32 == ok => Ok(()),
            error if ffi::SQLITE_ERROR as i32 == error => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Shutdown error",
            ))),
            other @ _ => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unknown shutdown error {other}"),
            ))),
        }
    }
}

#[test]
pub fn test_init() -> Result<(), self::Error> {
    sqlite3_init(1)?;
    sqlite3_shutdown()?;
    Ok(())
}



use std::{ptr, sync::Arc};
use tokio::sync::{Semaphore, oneshot};
use crossbeam::queue::ArrayQueue; // High-performance lock-free queue
use std::ffi::CString;

// 1. Your "Promise" to the compiler
// Not Sync! We want exclusive ownership only.



pub unsafe fn open_native_sqlite(path: &str) -> *mut crate::ffi::sqlite3 {
    const OPEN_FLAGS: i32 = (crate::ffi::SQLITE_OPEN_READWRITE | 
                        crate::ffi::SQLITE_OPEN_CREATE | 
                        crate::ffi::SQLITE_OPEN_NOMUTEX) as i32;

    let mut db_ptr: *mut crate::ffi::sqlite3 = core::ptr::null_mut();
    let c_path = CString::new(path).expect("Invalid path string");

    // sqlite3_open_v2 returns an integer status code
    
    let result = unsafe {crate::ffi::sqlite3_open_v2(
        c_path.as_ptr(),
        &mut db_ptr,
        OPEN_FLAGS,
        core::ptr::null(), // Use default VFS
    )};

    if result != crate::ffi::SQLITE_OK as i32 {
        // Handle error: result code 0 is SQLITE_OK
        panic!("Failed to open SQLite: {}", result);
    }

    db_ptr
}


// #[tokio::test]
// async fn test_parallel_queries() {
//     sqlite3_init(1).unwrap();
//     let n_tasks = 100_000;
//     let pool_size = 10;
    
//     // 2. Pre-allocate the pool (No allocations in the hot path)
//     let pool = Arc::new(ArrayQueue::new(pool_size));
//     for _ in 0..pool_size {
//         let conn = unsafe { open_native_sqlite(":memory:") }; 
//         pool.push(SqliteContext{handle:conn}).unwrap();
//     }

//     // 3. The Gatekeeper (Throttles tasks to pool size)
//     let semaphore = Arc::new(Semaphore::new(pool_size));
//     let mut handlers = vec![];
//     let start_time = std::time::Instant::now();
//     for i in 0..n_tasks {
//         let sem = semaphore.clone();
//         let p = pool.clone();

//         let h = tokio::spawn(async move {
//             // Wait for a slot
//             let _permit = sem.acquire().await.unwrap();
            
//             // "Take" from the pool (Lock-free pop)
//             let ctx = p.pop().expect("Semaphore guaranteed a context");

//             // 4. The FFI Call (Native-optimized)
//             let result = unsafe {
//                 execute_and_get_int(ctx.handle, r#"WITH RECURSIVE 
//   a(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM a LIMIT 50000),
//   b(y) AS (SELECT 1 UNION ALL SELECT y+1 FROM b LIMIT 50000)
// SELECT COUNT(*) FROM a, b 
// WHERE a.x = b.y AND (a.x % 500 = 0);"#)
//             };

//             // println!("Task {}: Result = {}", i, result);

//             // 5. "Recycle" (Lock-free push)
//             p.push(ctx).ok(); 
//             // _permit drops here, waking next task
//         });
//         handlers.push(h);
//     }

//     for h in handlers { h.await.unwrap(); }

//     let duration = start_time.elapsed();
//     dbg!(duration);


//     sqlite3_shutdown().unwrap();
// }

#[tokio::test]
async fn test_parallel_queries_multicore() {
    // 1. Setup Architecture
    let total_tasks = 100_000;
    let core_count = num_cpus::get(); // Detect physical/logical cores
    let tasks_per_core = total_tasks / core_count;
    let pool_size_per_core = 10; // Each core gets its own 10 connections

    // Global init once
    unsafe { sqlite3_init(0).unwrap(); }

    let start_time = std::time::Instant::now();
    let mut thread_handles = vec![];
    
    for core_id in 0..core_count {
        // Spawn a dedicated OS thread per core
        let handle = std::thread::spawn(move || {
            // 2. Hardware Pinning (Windows)
            // Ensures this thread NEVER leaves its assigned core
            affinity::set_thread_affinity([core_id]).unwrap();
            
            // 3. Dedicated Local Runtime
            let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        
        rt.block_on(async {
            let local = tokio::task::LocalSet::new();
            
            // 4. Private Pool for THIS core (No Arcs, No Mutexes needed)
            let pool = std::rc::Rc::new(ArrayQueue::new(pool_size_per_core));
            for _ in 0..pool_size_per_core {
                let conn = unsafe { open_native_sqlite(":memory:") };
                let mut context = SqliteContext::from_initialized_without_prepared_statement(conn);
                context.prepare_sql( r#"WITH RECURSIVE a(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM a LIMIT 50000), b(y) AS (SELECT 1 UNION ALL SELECT y+1 FROM b LIMIT 50000) SELECT COUNT(*) FROM a, b WHERE a.x = b.y AND (a.x % 500 = 0);"#);
                    // prepare
                    
                    pool.push(context).unwrap();
                }
                
                let semaphore = std::sync::Arc::new(Semaphore::new(pool_size_per_core));
                
                local.run_until(async {
                    let mut tasks = vec![];
                    for _ in 0..tasks_per_core {
                        let sem = semaphore.clone();
                        let p = pool.clone();
                        
                        // Spawn_local ensures the task stays on this core
                        tasks.push(tokio::task::spawn_local(async move {
                            let _permit = sem.acquire().await.unwrap();
                            let mut ctx = p.pop().expect("Private pool guaranteed");

                            unsafe {
                                execute_and_get_int(&mut ctx);
                            }

                            p.push(ctx).ok();
                        }));
                    }
                    for t in tasks { t.await.unwrap(); }
                }).await;
            });
        });
        thread_handles.push(handle);
    }

    // Wait for all cores to finish
    for h in thread_handles { h.join().unwrap(); }

    let duration = start_time.elapsed();
    println!("\n--- Multi-Core Results ---");
    println!("Total Time: {:?}", duration);
    println!("Throughput: {:.2} tasks/sec", total_tasks as f64 / duration.as_secs_f64());

    unsafe { sqlite3_shutdown().unwrap(); }
}




// Minimal FFI helpers for the example
unsafe fn execute_and_get_int(ctx: &mut SqliteContext) -> i32 {
    // In your real code, bindgen generates these
    // Your Stage 3 compiler will inline the transition to C here
     // Simulated return for the select 1
    ctx.reset_stmt();

    let mut result = 0;
    if unsafe{self::ffi::sqlite3_step(ctx.stmt)} == self::ffi::SQLITE_ROW as i32 {
        result = unsafe {self::ffi::sqlite3_column_int(ctx.stmt, 0)};
    }
    result


}