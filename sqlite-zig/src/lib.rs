// any core ffi functions that can frequently change or need grouping go here
// otherwise put the ffi interfaces in the place where they are used with feature gates

pub mod ffi;
pub mod alloc;
pub mod config;

pub type Error = Box<dyn std::error::Error>;


pub fn sqlite_zig_init(parameters: u32) -> Result<(),self::Error> {
    unsafe {
        sqlite_zig_configure(parameters)?;
        let r = ffi::sqlite_zig_init();
        match r {
            // ffi::SQliteZigInitResult::Ok => Ok(()),
            ffi::SQliteZigInitResult::Error => {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Initialization error")))},
            ffi::SQliteZigInitResult::UnknownError => {return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Unknown initialization error")));}
            _ =>{}
        }
        Ok(())
    }
}

fn sqlite_zig_configure(parameters: u32) -> Result<(),self::Error> {
    unsafe {
        let r = ffi::sqlite_zig_configure(parameters);
        match r {
            ffi::SqliteConfigResult::Ok => Ok(()),
            ffi::SqliteConfigResult::Error => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Configuration error"))),
            ffi::SqliteConfigResult::UnknownError => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Unknown configuration error"))),
        }
    }
}


pub fn sqlite_zig_shutdown() -> Result<(),self::Error> {
    unsafe {
        let r = ffi::sqlite_zig_shutdown();
        match r {
            ffi::SQliteZigShutdownResult::Ok => Ok(()),
            ffi::SQliteZigShutdownResult::Error => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Shutdown error"))),
            ffi::SQliteZigShutdownResult::UnknownError => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Unknown shutdown error"))),
        }
    }
}

#[test]
pub fn test_init() -> Result<(),self::Error> {
    sqlite_zig_init(1)
}