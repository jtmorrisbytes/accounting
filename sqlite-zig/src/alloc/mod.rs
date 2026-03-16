/// ALLOCATOR API even though they are PUB, they are not meant to be called by rust code
/// only by the underlying library
extern "C" fn __rust_sqlite_zig_alloc_malloc(n:i32) -> core::ffi::c_void {}
extern "C" fn __rust_sqlite_zig_alloc_free(p: core::ffi::c_void) {}
extern "C" fn __rust_sqlite_zig_alloc_realloc(p: core::ffi::c_void) {}
extern "C" fn __rust_sqlite_zig_alloc_size(p: core::ffi::c_void) {}
extern "C" fn __rust_sqlite_zig_alloc_roundup(p: core::ffi::c_void) {}
extern "C" fn __rust_sqlite_zig_alloc_init(p: core::ffi::c_void) {}
extern "C" fn __rust_sqlite_zig_alloc_shutdown(p: core::ffi::c_void) {}