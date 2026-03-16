const sqlite = @cImport({
    @cInclude("sqlite.h");
});
// mem management callback functions

// Zig promises these exist elsewhere (in the Rust binary)

// the purpose of these functions is to delegate the allocations to the
// rust specific allocator, unifying memory access

extern fn __rust_sqlite_zig_alloc_malloc(n: i32) callconv(.c) ?*anyopaque;
extern fn __rust_sqlite_zig_alloc_free(p: ?*anyopaque) callconv(.c) void;
extern fn __rust_sqlite_zig_alloc_realloc(p: ?*anyopaque, n: i32) callconv(.c) ?*anyopaque;
extern fn __rust_sqlite_zig_alloc_size(p: ?*anyopaque) callconv(.c) i32;
extern fn __rust_sqlite_zig_alloc_roundup(n: i32) callconv(.c) i32;
extern fn __rust_sqlite_zig_alloc_init(p: ?*anyopaque) callconv(.c) i32;
extern fn __rust_sqlite_zig_alloc_shutdown(p: ?*anyopaque) callconv(.c) void;

pub fn get_mem_methods() sqlite.sqlite3_mem_methods {
    return .{
        .xMalloc = __rust_sqlite_zig_alloc_malloc,
        .xFree = __rust_sqlite_zig_alloc_free,
        .xRealloc = __rust_sqlite_zig_alloc_realloc,
        .xSize = __rust_sqlite_zig_alloc_size,
        .xRoundup = __rust_sqlite_zig_alloc_roundup,
        .xInit = __rust_sqlite_zig_alloc_init,
        .xShutdown = __rust_sqlite_zig_alloc_shutdown,
        .pAppData = null,
    };
}
