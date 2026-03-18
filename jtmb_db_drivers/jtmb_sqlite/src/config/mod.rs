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


fn _jtmb_sqlite_comptime_threadsafe() -> Result<i32, crate::Error> {
    unsafe {
        match crate::ffi::sqlite3_threadsafe() {
            val @ 0..=2 => Ok(val),
            other => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("jtmb_sqlite3_comptime_threadsafe returned unknown val {other}"),
            )
            .into()),
        }
    }
}

fn _jtmb_sqlite_configure_singlethreaded() -> Result<(), crate::Error> {
    let r = unsafe { crate::ffi::sqlite3_config(crate::ffi::SQLITE_CONFIG_SINGLETHREAD.try_into()?) };
    Ok(())
}

fn _jtmb_sqlite_configure_multithreaded() -> Result<(), crate::Error> {
    unsafe {
        let r = crate::ffi::sqlite3_config(crate::ffi::SQLITE_CONFIG_MULTITHREAD.try_into()?);
    }
    Ok(())
}
fn _jtmb_sqlite_configure_malloc() -> Result<(), crate::Error> {
    unsafe {
        let r = crate::ffi::sqlite3_config(crate::ffi::SQLITE_CONFIG_MALLOC.try_into()?,crate::alloc::get_mem_methods());
        Ok(())
    }
}
fn _jtmb_sqlite_configure_memstatus(boolean: bool) -> Result<(), crate::Error> {
    let r = unsafe { crate::ffi::sqlite3_config(crate::ffi::SQLITE_CONFIG_MEMSTATUS.try_into()?, boolean as core::ffi::c_int ) };
    Ok(())
}

fn _jtmb_sqlite_configure_pagecache(size: usize) -> Result<(), crate::Error> {
    unsafe {
        // let _r = sqlite3_configure_pagecache(size);
        Ok(())
    }
}

fn _jtmb_sqlite_enable_uris(boolean: bool) -> Result<(),crate::Error> {
    let _r = unsafe {crate::ffi::sqlite3_config(crate::ffi::SQLITE_CONFIG_URI.try_into()?, boolean as core::ffi::c_int)};
    Ok(())
}

pub fn jtmb_sqlite_configure(parameters: u32) -> Result<(), crate::Error> {
    // defualt "singlethreaded. we turn off all safeguards and ensure we program correctly "
    let threading = _jtmb_sqlite_comptime_threadsafe()?;
    // we purposely turn off mutexes, stack canaries, and the like. our code needs to work without them
    if threading > 0 {
        _jtmb_sqlite_configure_singlethreaded()?;
    }
    _jtmb_sqlite_configure_malloc()?;
    _jtmb_sqlite_configure_memstatus(false)?;
    _jtmb_sqlite_configure_pagecache(64 * 1024 * 1024)?;
    _jtmb_sqlite_enable_uris(true)?;
    Ok(())
}
