const std = @import("std");

pub fn build(b: *std.Build) void {
    // Step to initialize git submodules
    const init_submodules = b.addSystemCommand(&[_][]const u8{
        "git",
        "submodule",
        "update",
        "--init",
        "--recursive",
    });

    // Make this the default step
    b.getInstallStep().dependOn(&init_submodules.step);
}
