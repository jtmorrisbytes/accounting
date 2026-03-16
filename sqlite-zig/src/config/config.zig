// handles configuring the underlying library

const sqlite = @cImport({
    @cInclude("sqlite3.h");
});
const alloc = @import("../alloc/alloc.zig");

const SQLiteZigConfigResult = enum(u32) {
    Success = 0,
    InvalidParameter = 1,
    UnsupportedOption = 2,
    UnknownError = 0xFFFFFFFF, // Catch-all for unexpected return values
};

/// for sqlite3 mem management via configure_malloc

// configures for multithreaing, uses guards and mutexes.
export fn sqlite_zig_configure_singlethreaded() void {
    const r = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_SINGLETHREAD);
    _ = r;
}
export fn sqlite_zig_configure_multithreaded() SQLiteZigConfigResult {
    const r = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_MULTITHREAD);
    const s = switch (r) {
        sqlite.SQLITE_OK => SQLiteZigConfigResult.Success,
        sqlite.SQLITE_ERROR => SQLiteZigConfigResult.UnsupportedOption,
        sqlite.SQLITE_MISUSE => SQLiteZigConfigResult.InvalidParameter,
        else => SQLiteZigConfigResult.UnknownError,
    };
    return s;
}

export fn sqlite_zig_configure_serialized() SQLiteZigConfigResult {
    const r = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_SERIALIZED);
    const s = switch (r) {
        sqlite.SQLITE_OK => SQLiteZigConfigResult.Success,
        sqlite.SQLITE_ERROR => SQLiteZigConfigResult.UnsupportedOption,
        sqlite.SQLITE_MISUSE => SQLiteZigConfigResult.InvalidParameter,
        else => SQLiteZigConfigResult.UnknownError,
    };
    return s;
}
export fn sqlite_zig_configure_malloc() void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_MALLOC, &alloc.get_mem_methods());
}

export fn sqlite_zig_configure_scratch(buffer: [*]u8, size: i32, min_alloc: i32) void {
    _ = buffer;
    _ = size;
    _ = min_alloc;
}

export fn sqlite_zig_configure_pagecache() void {}

// export fn sqlite_zig_configure_heap() void {}

export fn sqlite_zig_configure_memstatus(boolean: bool) void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_MEMSTATUS, boolean);
}

export fn sqlite_zig_configure_mutex() void {}

export fn sqlite_zig_configure_getmutex() void {}

export fn sqlite_zig_configure_log() void {}

export fn sqlite_zig_configure_lookaside() void {}

/// accepts configuration options via bitwise parameters.
export fn sqlite_zig_configure(parameters: u32) u32 {
    _ = parameters;
    return 0;
}
