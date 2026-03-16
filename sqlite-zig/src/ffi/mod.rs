




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

#[link(name = "jtmb_sqlite", kind = "static")]
unsafe extern "C" {
    pub unsafe fn jtmb_sqlite_init() -> SQliteZigInitResult;
    pub unsafe fn jtmb_sqlite_shutdown() -> SQliteZigShutdownResult;
}
