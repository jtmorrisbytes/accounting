const std = @import("std");

pub fn build(b: *std.Build) !void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // 1. Resolve the sibling 'sqlite-zig' package
    // This looks at your build.zig.zon for the .sqlite_zig path
    const sqlite_dep = b.dependency("sqlite_zig", .{
        .target = target,
        .optimize = optimize,
    });

    // 2. PLUCK THE ARTIFACT: Get the 'sqlite_driver' from the sub-package
    // Ensure 'sqlite_driver' matches the name in sqlite-zig/build.zig
    const driver_lib = sqlite_dep.artifact("sqlite_zig");

    // 3. INSTALL IT: Force the .lib into the root's zig-out/lib folder
    const install_driver = b.addInstallArtifact(driver_lib, .{
        .dest_dir = .{ .override = .lib },
        .pdb_dir = .default,
        .h_dir = .default,
    });

    // 4. Build your 'sql' crate using Stage 2 Nightly

    // B. The global 'install' step (default 'zig build') must wait for Cargo
    b.getInstallStep().dependOn(&install_driver.step);
}
