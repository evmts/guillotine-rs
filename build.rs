//! Build script to compile guillotine-mini Zig library
//!
//! Note: Currently builds WASM only. Future work will add native library support
//! or use wasmtime for execution.

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

    // WASM output is in zig-out/bin/guillotine_mini.wasm
    // TODO: Either add native build target or integrate with wasmtime
    eprintln!("guillotine-mini WASM built: {}/zig-out/bin/guillotine_mini.wasm",
              guillotine_mini_dir.display());
}
