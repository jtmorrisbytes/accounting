use libsqlite3_sys::*;
// borrows the slice of data from the internal library pointer. it is NOT SAFE
// to hold the slice of this funcion longer than 'slice or after sqlite3_step
pub unsafe fn stmt_txt_to_slice_u8<'slice>(
    stmt: *mut sqlite3_stmt,
    col_index: i32,
) -> Option<&'slice [u8]> {
    let text = unsafe { sqlite3_column_text(stmt, col_index) };
    let len = unsafe { sqlite3_column_bytes(stmt, col_index) };
    if text.is_null() {
        return None;
    }
    Some(unsafe { std::slice::from_raw_parts(text, len as usize) })
}

pub unsafe fn stmt_text_to_vec_u8(stmt: *mut sqlite3_stmt, col_index: i32) -> Option<Vec<u8>> {
    unsafe { stmt_txt_to_slice_u8(stmt, col_index) }.map(|slice| slice.to_vec())
}
pub unsafe fn stmt_text_to_string_lossy_empty_if_null(
    stmt: *mut sqlite3_stmt,
    col_index: i32,
) -> String {
    let slice = unsafe { stmt_txt_to_slice_u8(stmt, col_index) };
    String::from_utf8_lossy(slice.unwrap_or_default()).to_string()
}
pub unsafe fn stmt_text_to_string_err_if_null(
    stmt: *mut sqlite3_stmt,
    col_index: i32,
) -> std::io::Result<String> {
    unsafe { stmt_txt_to_slice_u8(stmt, col_index) }
        .map(|slice| String::from_utf8_lossy(slice).to_string())
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "sqlite3 returned a null pointer while attempting to extract text",
            )
        })
}
// gets the column name from the current row
// it is UNSAFE to hold the reference between sqlite_step calls
pub unsafe fn sqlite3_column_name_cstr<'slice>(
    stmt: *mut sqlite3_stmt,
    col_index: i32,
) -> Option<&'slice std::ffi::CStr> {
    let ptr = unsafe { sqlite3_column_name(stmt, col_index) };
    let _size = unsafe { sqlite3_column_bytes(stmt, col_index) };
    if ptr.is_null() {
        return None;
    } else {
        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr) };
        Some(c_str)
    }
}
pub fn sqlite3_column_name_string(stmt: *mut sqlite3_stmt, col_index: i32) -> String {
    let cstr = unsafe { sqlite3_column_name_cstr(stmt, col_index) };
    cstr.map(|cstr| cstr.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn extract_f64_from_statement_checked(
    stmt: *mut sqlite3_stmt,
    col_index: i32,
) -> std::io::Result<f64> {
    if stmt.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "attemtpted to call extract_f64_from_statement_checked with a null statement pointer"
            ),
        ));
    }

    let num_columns = unsafe { sqlite3_column_count(stmt) };
    if col_index > num_columns {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Attempted to call extract_f64_from_statement_checked with a column index {col_index} > num_columns {num_columns}"
            ),
        ));
    }
    let col_type = unsafe { sqlite3_column_type(stmt, col_index) };
    if col_type != SQLITE_FLOAT {
        let name = sqlite3_column_name_string(stmt, col_index);

        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("The provided column index for name '{name}' did not match SQLITE_FLOAT "),
        ));
    }
    let t = unsafe {sqlite3_column_double(stmt, col_index)};
    Ok(t)
}
pub unsafe fn extract_f64_from_statement_unchecked(stmt: *mut sqlite3_stmt,col_index: i32) -> f64 {
    debug_assert!(stmt.is_null() == false);
    unsafe {sqlite3_column_double(stmt, col_index)}
}

pub  fn extract_f32_from_statement_checked(stmt: *mut sqlite3_stmt, col_index: i32) -> std::io::Result<f32> {
    extract_f64_from_statement_checked(stmt, col_index).map(|float| float as f32)
}
// pub  fn extract_f16_from_statement_truncated_checked(stmt: *mut sqlite3_stmt, col_index: i32) -> std::io::Result<f32> {
//     extract_f64_from_statement_checked(stmt, col_index).map(|float| f64::min(float,f16::MAX as f64) as f32)
// }
// pub unsafe fn extract_f128_from_statement_checked(stmt: *mut sqlite3_stmt,col_index: i32) -> std::io::Result<f128> {
//     extract_f64_from_statement_checked(stmt, col_index).map(|f| f as f128)
// }

pub fn extract_f32_from_statement_unchecked(stmt: *mut sqlite3_stmt, col_index: i32) -> f32{
    unsafe {extract_f64_from_statement_unchecked(stmt, col_index) as f32}
}

pub fn extract_integer_from_statement_checked(stmt: *mut sqlite3_stmt,col_index: i32) -> std::io::Result<i64> {
    if stmt.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "attemtpted to call extract_integer_from_statement_checked with a null statement pointer"
            ),
        ));
    }
    let num_columns = unsafe { sqlite3_column_count(stmt) };
    if col_index > num_columns {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Attempted to call extract_f64_from_statement_checked with a column index {col_index} > num_columns {num_columns}"
            ),
        ));
    }
    let col_type = unsafe { sqlite3_column_type(stmt, col_index) };
    if col_type != SQLITE_INTEGER {
        let name = sqlite3_column_name_string(stmt, col_index);

        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("The provided column index for name '{name}' did not match SQLITE_FLOAT "),
        ));
    }
    let t = unsafe {sqlite3_column_int64(stmt, col_index)};
    Ok(t)

}

pub fn extract_integer_from_statement(stmt: *mut sqlite3_stmt,col_index: i32) -> std::io::Result<i64> {
    #[cfg(debug_assertions)] {
        extract_integer_from_statement_checked(stmt, col_index)
    }
    #[cfg(not(debug_assertions))] {
        Ok(extract_integer_from_statement_unchecked(stmt, col_index))
    }
}

/// it is NOT SAFE to hold this beyond 'slice or sqlite3_step
pub unsafe fn extract_blob_from_statement_checked<'slice>(stmt: *mut sqlite3_stmt,col_index: i32) -> std::io::Result<&'slice [u8]> {
       if stmt.is_null() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "attemtpted to call extract_blob_from_statement_checked with a null statement pointer"
            ),
        ));
    }
    let num_columns = unsafe { sqlite3_column_count(stmt) };
    if col_index > num_columns {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Attempted to call extract_blob_from_statement_checked with a column index {col_index} > num_columns {num_columns}"
            ),
        ));
    }
    let col_type = unsafe { sqlite3_column_type(stmt, col_index) };
    if col_type != SQLITE_BLOB {
        let name = sqlite3_column_name_string(stmt, col_index);

        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("The provided column index for name '{name}' did not match SQLITE_FLOAT "),
        ));
    }
    let t = unsafe {sqlite3_column_blob(stmt, col_index)};
    let len = unsafe {sqlite3_column_bytes(stmt, col_index)};
    let slice = unsafe {std::slice::from_raw_parts(t as *const u8, len as usize)};
    Ok(slice)
}
pub fn extract_owned_blob_from_statement_checked(stmt: *mut sqlite3_stmt,col_index: i32) -> std::io::Result<Vec<u8>> {
    unsafe {extract_blob_from_statement_checked(stmt, col_index)}.map(|slice|slice.to_vec())
}
pub fn extract_blob_from_statement_unchecked<'life>(stmt: *mut sqlite3_stmt, col_index: i32) -> &'life[u8] {
    debug_assert!(stmt.is_null() == false);
    let t = unsafe {sqlite3_column_blob(stmt, col_index)};
    debug_assert!(t.is_null() == false);
    let len = unsafe {sqlite3_column_bytes(stmt, col_index)};
    let slice = unsafe {std::slice::from_raw_parts(t as *const u8, len as usize)};
    slice
} 