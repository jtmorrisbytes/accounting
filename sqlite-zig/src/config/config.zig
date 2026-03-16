// handles configuring the underlying library

const sqlite = @cImport({
    @cInclude("sqlite.h");
});

const SQLiteZigConfigResult = enum(u32) {
    Success = 0,
    InvalidParameter = 1,
    UnsupportedOption = 2,
    UnknownError = 0xFFFFFFFF, // Catch-all for unexpected return values
};

/// for sqlite3 mem management via configure_malloc

// configures for multithreaing, uses guards and mutexes.
export fn sqlite_zig_v0_configure_singlethreaded() void {
    const r = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_SINGLETHREAD);
    _ = r;
}
export fn sqlite_zig_v0_configure_multithreaded() void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_MULTITHREAD);
}

export fn sqlite_zig_v0_configure_serialized() void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_SERIALIZED);
}
export fn sqlite_zig_v0_configure_malloc() void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIG_MALLOC, &sqlite_zig_alloc.get_mem_methods());
}

export fn sqlite_zig_v0_configure_scratch(buffer: [*]u8, size: i32, min_alloc: i32) void {
    _ = buffer;
    _ = size;
    _ = min_alloc;
}

export fn sqlite_zig_v0_configure_pagecache() void {}

// export fn sqlite_zig_v0_configure_heap() void {}

export fn sqlite_zig_v0_configure_memstatus(boolean: bool) void {
    _ = sqlite.sqlite3_config(sqlite.SQLITE_CONFIGURE_MEMSTATUS, boolean);
}

export fn sqlite_zig_v0_configure_mutex() void {}

export fn sqlite_zig_v0_configure_getmutex() void {}

export fn sqlite_zig_v0_configure_log() void {}

export fn sqlite_zig_v0_configure_lookaside() void {}

/// accepts configuration options via bitwise parameters.
export fn sqlite_zig_v0_configure(parameters: u32) u32 {
    _ = parameters;
    return 0;
}
