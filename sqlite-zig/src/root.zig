//! By convention, root.zig is the root source file when making a library.
// const std = @import("std");
const sqlite = @cImport({
    @cInclude("sqlite3.h");
});
/// imports the public api
pub const sqlite_zig_alloc = @import("alloc/alloc.zig");
pub const sqlite_zig_config = @import("config/config.zig");
comptime {
    _ = sqlite_zig_alloc;
    _ = sqlite_zig_config;
}
const SQLiteZigInitResult = enum(c_int) {
    Ok = 0,
    Error = 1,
    UnknownError = -1, // Catch-all for unexpected return values
    // Add more specific error codes as needed
};

// initializes the sqlite library. This should be called before using any other SQLite functions. It sets up internal data structures and prepares the library for use.
export fn sqlite_zig_init() SQLiteZigInitResult {
    const retval = sqlite.sqlite3_initialize();
    const r = switch (retval) {
        sqlite.SQLITE_OK => SQLiteZigInitResult.Ok,
        sqlite.SQLITE_ERROR => SQLiteZigInitResult.Error,
        else => SQLiteZigInitResult.UnknownError,
    };
    return r;
}

const SQliteZigShutdownResult = enum(i32) {
    Ok = 0,
    Error = 1,
    UnknownError = -1, // Catch-all for unexpected return values
    // Add more specific error codes as needed
};

/// de-initializes the SQLite library. This should be called when your application is done using SQLite, typically just before exiting.
export fn sqlite_zig_shutdown() SQliteZigShutdownResult {
    const retval = sqlite.sqlite3_shutdown();
    const r = switch (retval) {
        sqlite.SQLITE_OK => SQliteZigShutdownResult.Ok,
        sqlite.SQLITE_ERROR => SQliteZigShutdownResult.Error,
        else => SQliteZigShutdownResult.UnknownError,
    };
    return r;
}
