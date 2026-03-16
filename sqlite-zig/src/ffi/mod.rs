




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

#[link(name = "sqlite_zig", kind = "static")]
unsafe extern "C" {
    pub unsafe fn sqlite_zig_init() -> SQliteZigInitResult;
    pub unsafe fn sqlite_zig_shutdown() -> SQliteZigShutdownResult;
}
