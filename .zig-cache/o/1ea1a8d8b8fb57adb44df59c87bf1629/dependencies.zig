pub const packages = struct {
    pub const @"sqlite-zig" = struct {
        pub const build_root = "C:\\Users\\jthec\\code\\jtmb\\sqlite-zig";
        pub const build_zig = @import("sqlite-zig");
        pub const deps: []const struct { []const u8, []const u8 } = &.{
        };
    };
};

pub const root_deps: []const struct { []const u8, []const u8 } = &.{
    .{ "sqlite_zig", "sqlite-zig" },
};
