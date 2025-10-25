const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Build just the Zig dependencies (used by cargo build.rs)
    const build_deps_step = b.step("build-deps", "Build only Zig dependencies (for cargo)");
    const lib = buildZigDeps(b, target, optimize);
    build_deps_step.dependOn(&b.addInstallArtifact(lib, .{}).step);

    // Default build: build Zig deps + run cargo build
    const cargo_step = b.addSystemCommand(&.{"cargo", "build"});
    cargo_step.step.dependOn(&b.addInstallArtifact(lib, .{}).step);
    b.getInstallStep().dependOn(&cargo_step.step);
}

fn buildZigDeps(
    b: *std.Build,
    target: std.Build.ResolvedTarget,
    optimize: std.builtin.OptimizeMode,
) *std.Build.Step.Compile {
    // Fetch guillotine-mini dependency
    const guillotine_mini_dep = b.dependency("guillotine_mini", .{
        .target = target,
        .optimize = optimize,
    });

    // Fetch primitives dependency directly
    const primitives_dep = b.dependency("guillotine_primitives", .{
        .target = target,
        .optimize = optimize,
    });

    // Get the primitives modules
    const primitives_mod = primitives_dep.module("primitives");
    const crypto_mod = primitives_dep.module("crypto");
    const precompiles_mod = primitives_dep.module("precompiles");

    // Create module for the C interface
    const root_c_mod = b.createModule(.{
        .root_source_file = guillotine_mini_dep.path("src/root_c.zig"),
        .target = target,
        .optimize = optimize,
        .imports = &.{
            .{ .name = "primitives", .module = primitives_mod },
            .{ .name = "crypto", .module = crypto_mod },
            .{ .name = "precompiles", .module = precompiles_mod },
        },
    });

    // Create a static library
    const lib = b.addLibrary(.{
        .name = "guillotine_mini",
        .root_module = root_c_mod,
        .linkage = .static,
    });

    // Link the crypto library from primitives
    lib.linkLibrary(primitives_dep.artifact("crypto_wrappers"));

    return lib;
}
