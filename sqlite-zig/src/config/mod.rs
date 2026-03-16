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


#[link(name = "sqlite_zig",kind="static")]
unsafe extern "C"  {

    // configures for multithreaing, uses guards and mutexes.
    pub(crate) fn sqlite_zig_configure_singlethreaded();
    pub(crate) fn sqlite_zig_configure_multithreaded();

    pub(crate) fn sqlite_zig_configure_serialized();
    pub(crate) fn sqlite_zig_configure_malloc();
    
    pub(crate) fn sqlite_zig_configure_scratch(buffer: *mut *mut u8, size: i32, min_alloc: i32);
    
    pub(crate) fn sqlite_zig_configure_pagecache(size:usize) -> i32;
    
    // pub(crate) fn sqlite_zig_v0_configure_heap();
    
    pub(crate) fn sqlite_zig_configure_memstatus(boolean: bool);
    
    pub(crate) fn sqlite_zig_configure_mutex();
    
    pub(crate) fn sqlite_zig_configure_getmutex();
    
    pub(crate) fn sqlite_zig_configure_log();
    
    pub(crate) fn sqlite_zig_configure_lookaside();
    /// asks if sqlite was compiled SQLITE_THREADSAFE
    pub(crate) unsafe fn sqlite_zig_comptime_threadsafe() -> i32;

    
    // accepts configuration options via bitwise parameters.
    // pub(crate) fn sqlite_zig_configure(parameters: u32) -> u32;
}

fn _sqlite_zig_comptime_threadsafe() -> Result<i32,crate::Error> {
    unsafe {
        match sqlite_zig_comptime_threadsafe() {
            val @ 0..=2 => Ok(val),
            other=> Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("sqlite_zig_comptime_threadsafe returned unknown val {other}")).into())
        }
    }
}


fn _sqlite_zig_configure_singlethreaded() -> Result<(),crate::Error> {
      let r = unsafe {
      
            sqlite_zig_configure_singlethreaded()
    };
    Ok(())
}


fn _sqlite_zig_configure_multithreaded() -> Result<(),crate::Error> {
    unsafe {
        let r =sqlite_zig_configure_multithreaded();
    }
    Ok(())
}
fn _sqlite_zig_configure_malloc() -> Result<(), crate::Error> {
    unsafe {
        let r = sqlite_zig_configure_malloc();
        Ok(())
    }
}
fn _sqlite_zig_configure_memstatus(boolean:bool) -> Result<(),crate::Error> {
        let r = unsafe {sqlite_zig_configure_memstatus(boolean)};
        Ok(())
}

fn _sqlite_zig_configure_pagecache(size:usize) -> Result<(),crate::Error> {
    unsafe {
        let _r  = sqlite_zig_configure_pagecache(size);
        Ok(())
    }
}

pub fn sqlite_zig_configure(parameters: u32) -> Result<(),crate::Error> {
    
    // defualt "singlethreaded. we turn off all safeguards and ensure we program correctly "
    let threading = _sqlite_zig_comptime_threadsafe()?;
    // we purposely turn off mutexes, stack canaries, and the like. our code needs to work without them
    if threading > 0 {
        _sqlite_zig_configure_singlethreaded()?;
    }
    _sqlite_zig_configure_malloc()?;
    _sqlite_zig_configure_memstatus(false)?;
    _sqlite_zig_configure_pagecache(64 * 1024 * 1024)?;
    Ok(())
}