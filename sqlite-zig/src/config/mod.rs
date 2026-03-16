#[repr(u32)]
pub enum SqliteConfigResult {
    Ok = 0,
    Error = 1,
    UnknownError = u32::MAX,
}


#[repr(u32)] 
pub enum SQliteZigConfigParameters {
        ThreadSafe = 0b00000001,
        TempStoreInMemory = 0b00000010,
        OmitAutoInit = 0b00000100,
    }


#[link(name = "jtmb_sqlite",kind="static")]
unsafe extern "C"  {

    // configures for multithreaing, uses guards and mutexes.
    pub(crate) fn jtmb_sqlite_configure_singlethreaded();
    pub(crate) fn jtmb_sqlite_configure_multithreaded();

    pub(crate) fn jtmb_sqlite_configure_serialized();
    pub(crate) fn jtmb_sqlite_configure_malloc();
    
    pub(crate) fn jtmb_sqlite_configure_scratch(buffer: *mut *mut u8, size: i32, min_alloc: i32);
    
    pub(crate) fn jtmb_sqlite_configure_pagecache(size:usize) -> i32;
    
    // pub(crate) fn jtmb_sqlite_v0_configure_heap();
    
    pub(crate) fn jtmb_sqlite_configure_memstatus(boolean: bool);
    
    pub(crate) fn jtmb_sqlite_configure_mutex();
    
    pub(crate) fn jtmb_sqlite_configure_getmutex();
    
    pub(crate) fn jtmb_sqlite_configure_log();
    
    pub(crate) fn jtmb_sqlite_configure_lookaside();
    /// asks if sqlite was compiled SQLITE_THREADSAFE
    pub(crate) unsafe fn jtmb_sqlite_comptime_threadsafe() -> i32;

    
    // accepts configuration options via bitwise parameters.
    // pub(crate) fn jtmb_sqlite_configure(parameters: u32) -> u32;
}

fn _jtmb_sqlite_comptime_threadsafe() -> Result<i32,crate::Error> {
    unsafe {
        match jtmb_sqlite_comptime_threadsafe() {
            val @ 0..=2 => Ok(val),
            other=> Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("jtmb_sqlite_comptime_threadsafe returned unknown val {other}")).into())
        }
    }
}


fn _jtmb_sqlite_configure_singlethreaded() -> Result<(),crate::Error> {
      let r = unsafe {
      
            jtmb_sqlite_configure_singlethreaded()
    };
    Ok(())
}


fn _jtmb_sqlite_configure_multithreaded() -> Result<(),crate::Error> {
    unsafe {
        let r =jtmb_sqlite_configure_multithreaded();
    }
    Ok(())
}
fn _jtmb_sqlite_configure_malloc() -> Result<(), crate::Error> {
    unsafe {
        let r = jtmb_sqlite_configure_malloc();
        Ok(())
    }
}
fn _jtmb_sqlite_configure_memstatus(boolean:bool) -> Result<(),crate::Error> {
        let r = unsafe {jtmb_sqlite_configure_memstatus(boolean)};
        Ok(())
}

fn _jtmb_sqlite_configure_pagecache(size:usize) -> Result<(),crate::Error> {
    unsafe {
        let _r  = jtmb_sqlite_configure_pagecache(size);
        Ok(())
    }
}

pub fn jtmb_sqlite_configure(parameters: u32) -> Result<(),crate::Error> {
    
    // defualt "singlethreaded. we turn off all safeguards and ensure we program correctly "
    let threading = _jtmb_sqlite_comptime_threadsafe()?;
    // we purposely turn off mutexes, stack canaries, and the like. our code needs to work without them
    if threading > 0 {
        _jtmb_sqlite_configure_singlethreaded()?;
    }
    _jtmb_sqlite_configure_malloc()?;
    _jtmb_sqlite_configure_memstatus(false)?;
    _jtmb_sqlite_configure_pagecache(64 * 1024 * 1024)?;
    Ok(())
}