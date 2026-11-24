const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Cargo links the matching c-kzg and blst archives. Zig needs their headers.
    const c_kzg = b.createModule(.{
        .root_source_file = b.path("vendor/c-kzg/root.zig"),
        .target = target,
        .optimize = optimize,
        .link_libc = true,
    });
    c_kzg.addIncludePath(b.path("vendor/c-kzg/include"));

    const crypto = b.createModule(.{
        .root_source_file = b.path("vendor/primitives/crypto/root.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{.{ .name = "c_kzg", .module = c_kzg }},
    });
    crypto.addIncludePath(b.path("vendor/primitives/ffi"));

    const primitives = b.createModule(.{
        .root_source_file = b.path("vendor/primitives/primitives/root.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{.{ .name = "crypto", .module = crypto }},
    });
    crypto.addImport("primitives", primitives);

    const precompiles = b.createModule(.{
        .root_source_file = b.path("vendor/primitives/precompiles/root.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "primitives", .module = primitives },
            .{ .name = "crypto", .module = crypto },
        },
    });

    const lib = b.addLibrary(.{
        .name = "guillotine_mini",
        .linkage = .static,
        .root_module = b.createModule(.{
            .root_source_file = b.path("vendor/guillotine-mini/root_c.zig"),
            .target = target,
            .optimize = optimize,
            .link_libc = true,
            .pic = true,
            .imports = &.{
                .{ .name = "primitives", .module = primitives },
                .{ .name = "crypto", .module = crypto },
                .{ .name = "precompiles", .module = precompiles },
            },
        }),
    });
    // Rust links the Zig archive directly and needs Zig's integer runtime helpers.
    lib.bundle_compiler_rt = true;
    b.installArtifact(lib);
}
