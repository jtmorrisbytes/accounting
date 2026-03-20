
// 1. The "Contract"
type ExtractFn = unsafe fn(*mut ffi::sqlite3_stmt, i32, &mut Vec<u8>);

// 2. The "VTable" (Index 0 is unused)
static EXTRACT_TBL: [ExtractFn; 6] = [
    nop_extract,   // 0: Guard
    pack_int,      // 1: SQLITE_INTEGER
    pack_float,    // 2: SQLITE_FLOAT
    pack_text,     // 3: SQLITE_TEXT
    pack_blob,     // 4: SQLITE_BLOB
    pack_null,     // 5: SQLITE_NULL
];

pub unsafe fn nop_extract(p:*mut ffi::sqlite3_stmt, c:i32, v:&mut Vec<u8>){}
pub unsafe fn pack_int(p:*mut ffi::sqlite3_stmt,c:i32,v:&mut Vec<u8> ){
        unsafe {
        let f = sqlite3_column_int64(p, c);
        v.extend_from_slice(f.to_ne_bytes().as_slice());
    }

}
pub unsafe fn pack_float(p:*mut ffi::sqlite3_stmt,c:i32,v:&mut Vec<u8> ){
    unsafe {
        let f = sqlite3_column_double(p, c);
        v.extend_from_slice(f.to_ne_bytes().as_slice());
    }
}
pub unsafe fn pack_text(p:*mut ffi::sqlite3_stmt,c:i32,v:&mut Vec<u8> ){
    unsafe {
        let ptr = sqlite3_column_text(p, i);
        debug_assert!(ptr.is_null() == false);
        let len = sqlite3_column_bytes(p, i);
        let s = std::slice::from_raw_parts(ptr, len as usize);
        v.extend_from_slice((len as u32).to_ne_bytes().as_slice());
    }
}
pub unsafe fn pack_blob(p:*mut ffi::sqlite3_stmt,c:i32,v:&mut Vec<u8> ){
    unsafe {
        let ptr = sqlite3_column_blob(p, c);
        debug_assert!(ptr.is_null() == false);
        let len = sqlite3_column_bytes(p, c);
        let s = std::slice::from_raw_parts(ptr as *const u8, len as usize);
        v.extend_from_slice((len as u32).to_ne_bytes().as_slice());
        v.extend_from_slice(s);
    }
}
pub unsafe fn pack_null(p:*mut ffi::sqlite3_stmt,c:i32,v:&mut Vec<u8> ){
    v.extend_from_slice(0_u32);
}
