const std = @import("std");
const root = @import("root.zig");

/// Build: zig build -Doptimize=ReleaseFast
/// Artifact: zig-out/lib/libzuchu_render.a -> linked by the zuchu crate.
pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{ .preferred_optimize_mode = .ReleaseFast });

    const lib = b.addStaticLibrary(.{
        .name = "zuchu_render",
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
    });
    // Export the C ABI function for Rust FFI
    lib.linkLibC();
    lib.root_module.export_symbol("composite", root.composite);
    b.installArtifact(lib);

    const tests = b.addTest(.{ .root_source_file = b.path("src/root.zig"), .target = target, .optimize = optimize });
    const run_tests = b.addRunArtifact(tests);
    const test_step = b.step("test", "Run library tests");
    test_step.dependOn(&run_tests.step);
}
