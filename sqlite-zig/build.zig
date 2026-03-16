const std = @import("std");

// Although this function looks imperative, it does not perform the build
// directly and instead it mutates the build graph (`b`) that will be then
// executed by an external runner. The functions in `std.Build` implement a DSL
// for defining build steps and express dependencies between them, allowing the
// build runner to parallelize the build automatically (and the cache system to
// know when a step doesn't need to be re-run).
pub fn build(b: *std.Build) !void {
    // Standard target options allow the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});
    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});
    // It's also possible to define more custom flags to toggle optional features
    // of this build script using `b.option()`. All defined flags (including
    // target and optimize options) will be listed when running `zig build --help`
    // in this directory.

    const nmake_path = b.findProgram(&.{"nmake"}, &.{}) catch |err| {
        std.debug.print("ERROR: Could not find nmake in PATH. Is the MSVC env loaded? {any}\n", .{err});
        return err;
    };

    const host_os = b.graph.host.result.os.tag;

    const assemble_step = b.addSystemCommand(&.{ nmake_path, "/f", b.path("vendor/sqlite-c-3150300/Makefile.msc").getPath(b), "sqlite3.c" });
    // build sqlite3.c into a static library that we can link to.

    if (host_os == .windows) {
        // You must grab these from the 'b.graph.env_map' or the host's env
        const env = b.graph.env_map;
        assemble_step.setEnvironmentVariable("PATH", env.get("PATH") orelse "");
        assemble_step.setEnvironmentVariable("INCLUDE", env.get("INCLUDE") orelse "");
        assemble_step.setEnvironmentVariable("LIB", env.get("LIB") orelse "");

        // Optional: If nmake needs the VS version info
        if (env.get("VCINSTALLDIR")) |v| assemble_step.setEnvironmentVariable("VCINSTALLDIR", v);
    }
    const base_dir = b.path("vendor/sqlite-c-3150300/");
    assemble_step.cwd = base_dir;

    // 1. Get the environment map from the build graph
    // Tell Zig that the .c file is a generated artifact from the assemble_step
    // const generated_c = assemble_step.addOutputFileArg("sqlite3.c");
    // const generated_h = assemble_step.addOutputFileArg("sqlite3.h");

    const sql_mod = b.addModule("sqlite_zig", .{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });

    const lib = b.addLibrary(.{
        .name = "sqlite_zig",
        .linkage = .static,
        .root_module = sql_mod,
    });

    lib.addCSourceFile(.{
        .file = base_dir.path(b, "sqlite3.c"),
        .flags = &.{
            "-march=native", // Heavy lifting for your 5600X
            "-O3",
            "-DSQLITE_THREADSAFE=0",
            "-DSQLITE_TEMP_STORE=3", // Use your 48GB RAM for temp tables
        },
    });
    lib.addIncludePath(base_dir);
    lib.step.dependOn(&assemble_step.step);
    b.installArtifact(lib);
}
