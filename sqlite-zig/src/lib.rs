// any core ffi functions that can frequently change or need grouping go here
// otherwise put the ffi interfaces in the place where they are used with feature gates

pub mod ffi;
pub mod alloc;
pub mod config;

pub type Error = Box<dyn std::error::Error>;


pub fn jtmb_sqlite_init(parameters: u32) -> Result<(),self::Error> {
    unsafe {
        config::jtmb_sqlite_configure(parameters)?;
        let r = ffi::jtmb_sqlite_init();
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




pub fn jtmb_sqlite_shutdown() -> Result<(),self::Error> {
    unsafe {
        let r = ffi::jtmb_sqlite_shutdown();
        match r {
            ffi::SQliteZigShutdownResult::Ok => Ok(()),
            ffi::SQliteZigShutdownResult::Error => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Shutdown error"))),
            ffi::SQliteZigShutdownResult::UnknownError => Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other,"Unknown shutdown error"))),
        }
    }
}

#[test]
pub fn test_init() -> Result<(),self::Error> {
    jtmb_sqlite_init(1)?;
    jtmb_sqlite_shutdown()?;
    Ok(())
}