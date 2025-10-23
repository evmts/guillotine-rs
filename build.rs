//! Build script to compile guillotine-mini Zig library

use std::process::Command;
use std::path::PathBuf;
use std::env;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let guillotine_mini_dir = manifest_dir.join("lib/guillotine-mini");

    println!("cargo:rerun-if-changed=lib/guillotine-mini/src");
    println!("cargo:rerun-if-changed=lib/guillotine-mini/build.zig");

    // Check if submodules are initialized
    let primitives_dir = guillotine_mini_dir.join("lib/primitives");
    if !primitives_dir.join("src").exists() {
        eprintln!("Initializing guillotine-mini submodules...");
        let status = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .current_dir(&guillotine_mini_dir)
            .status()
            .expect("Failed to initialize submodules");

        if !status.success() {
            panic!("git submodule init failed");
        }
    }

    // Build guillotine-mini WASM library using zig
    let status = Command::new("zig")
        .args(&["build", "wasm"])
        .current_dir(&guillotine_mini_dir)
        .status()
        .expect("Failed to execute zig build");

    if !status.success() {
        panic!("zig build failed");
    }

    // Tell cargo where to find the library
    let lib_dir = guillotine_mini_dir.join("zig-out/lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=guillotine_mini");
}
