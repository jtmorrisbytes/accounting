
#[repr(i32)]
pub enum SQliteZigInitResult {
    Ok = 0,
    Error = 1,
    UnknownError = -1,
}

#[repr(i32)]
pub enum SQliteZigShutdownResult {
    Ok = 0,
    Error = 1,
    UnknownError = -1,
}

#[repr(u32)] 
pub enum SQliteZigConfigParameters {
        ThreadSafe = 0b00000001,
        TempStoreInMemory = 0b00000010,
        OmitAutoInit = 0b00000100,
    }

#[repr(u32)]
pub enum SqliteConfigResult {
    Ok = 0,
    Error = 1,
    UnknownError = u32::MAX,
}



#[link(name = "sqlite_zig", kind = "static")]
unsafe extern "C" {
    pub unsafe fn sqlite_zig_v0_init() -> SQliteZigInitResult;
    pub unsafe fn sqlite_zig_v0_shutdown() -> SQliteZigShutdownResult;
    pub unsafe fn sqlite_zig_v0_configure(parameters: u32) -> SqliteConfigResult;
}



