use libsqlite3_sys::{
    sqlite3_bind_double, sqlite3_bind_int64, sqlite3_bind_text, sqlite3_column_double,
    sqlite3_column_int64, sqlite3_destructor_type, sqlite3_finalize, sqlite3_prepare_v2,
    sqlite3_step, sqlite3_stmt,
};
use std::ops::Deref;
type Real = f64;
type Float = f64;
type Integer = i64;
type Text = String;
struct Blob(Vec<u8>);
impl Deref for Blob {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait BindTo {
    fn bind_to(self, col: i32, stmt: *mut sqlite3_stmt) -> i32;
}

// impl bind for types that are supported

impl BindTo for self::Integer {
    fn bind_to(self, col: i32, stmt: *mut sqlite3_stmt) -> i32 {
        unsafe { sqlite3_bind_int64(stmt, col, self) }
    }
}
impl BindTo for self::Float {
    fn bind_to(self, col: i32, stmt: *mut sqlite3_stmt) -> i32 {
        unsafe { sqlite3_bind_double(stmt, col, self) }
    }
}
impl BindTo for &str {
    fn bind_to(self, col: i32, stmt: *mut sqlite3_stmt) -> i32 {
        unsafe {
            let transient: sqlite3_destructor_type = std::mem::transmute(-1_isize);
            sqlite3_bind_text(
                stmt,
                col,
                self.as_ptr().cast(),
                self.len() as i32,
                transient,
            )
        }
    }
}
impl BindTo for String {
    fn bind_to(self, col: i32, stmt: *mut sqlite3_stmt) -> i32 {
        self.as_str().bind_to(col, stmt)
    }
}

pub trait ExtractFrom {
    fn extract_from(stmt: *mut sqlite3_stmt, col: i32) -> Self;
}
impl ExtractFrom for Integer {
    fn extract_from(stmt: *mut sqlite3_stmt, col: i32) -> self::Integer {
        unsafe { sqlite3_column_int64(stmt, col) }
    }
}
impl ExtractFrom for Float {
    fn extract_from(stmt: *mut sqlite3_stmt, col: i32) -> Self {
        unsafe { sqlite3_column_double(stmt, col) }
    }
}

#[repr(transparent)]
pub struct Statement(*mut sqlite3_stmt);

impl Statement {
    pub(crate) fn prepare<'conn>(connection: &'conn super::ConnectionHandle, sql: &str) -> Self {
        // let cstring = std::ffi::CString::new(sql).expect("Non null terminated string");
        let mut ptr = std::ptr::null_mut();
        unsafe {
            sqlite3_prepare_v2(
                **connection,
                sql.as_ptr(),
                sql.len() as i32,
                &mut ptr,
                std::ptr::null_mut(),
            );
        }
        Self(ptr)
    }

    pub(crate) fn step(&mut self) -> i32 {
        debug_assert!(self.0.is_null() == false);
        unsafe { sqlite3_step(self.0) }
    }
    pub(crate) fn text(&mut self, col: i32) -> String {
        unsafe { super::utils::stmt_text_to_string_lossy_empty_if_null(self.0, col) }
    }
    pub(crate) fn bind<T>(self, col: i32, val: T) -> Self
    where
        T: BindTo,
    {
        #[cfg(debug_assertions)]
        {
            let max = unsafe { libsqlite3_sys::sqlite3_bind_parameter_count(self.0) };
            if col < 1 || col > max {
                panic!(
                    "Bind Error: Column index {} is out of range (1..{})",
                    col, max
                );
            }
        }
        let rc = val.bind_to(col, self.0);
        debug_assert_eq!(rc, libsqlite3_sys::SQLITE_OK, "SQLite Bind failed with error code: {}", rc);
        self
    }
    pub(crate) fn extract<T>(&self, col: i32) -> T
    where
        T: ExtractFrom,
    {
        let t = T::extract_from(self.0, col);
        t
    }
}
// not sure if needed but DONT USE WITH MORE THAN 1 THREAD AT SAME TIME. DO NOT CLONE
unsafe impl Send for Statement {}

impl std::ops::Deref for Statement {
    type Target = *mut sqlite3_stmt;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Statement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Statement {
    fn drop(&mut self) {
        let _ = unsafe { sqlite3_finalize(self.0) };
    }
}
