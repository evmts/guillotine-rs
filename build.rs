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

    // Build guillotine-mini native static library using zig
    let status = Command::new("zig")
        .args(&["build", "native"])
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

    // Link the Rust crypto libraries (ARK BN254/BLS12-381 and Keccak)
    // These are built by guillotine-mini's build system in lib/primitives
    let primitives_lib_dir = guillotine_mini_dir.join("lib/primitives/target/release");

    if primitives_lib_dir.exists() {
        println!("cargo:rustc-link-search=native={}", primitives_lib_dir.display());

        // Link BN254 wrapper (contains BLS12-381 G1 symbols)
        if primitives_lib_dir.join("libbn254_wrapper.a").exists() {
            println!("cargo:rustc-link-lib=static=bn254_wrapper");
            eprintln!("ARK BN254 wrapper library found: {}/libbn254_wrapper.a", primitives_lib_dir.display());
        } else {
            eprintln!("WARNING: BN254 wrapper not found");
        }

        // Link Keccak wrapper (contains keccak256 symbol)
        if primitives_lib_dir.join("libkeccak_wrapper.a").exists() {
            println!("cargo:rustc-link-lib=static=keccak_wrapper");
            eprintln!("Keccak wrapper library found: {}/libkeccak_wrapper.a", primitives_lib_dir.display());
        } else {
            eprintln!("WARNING: Keccak wrapper not found");
        }
    } else {
        eprintln!("WARNING: Primitives library directory not found: {}", primitives_lib_dir.display());
        eprintln!("Crypto operations may not be available");
    }

    eprintln!("guillotine-mini native library built: {}/libguillotine_mini.a",
              lib_dir.display());
}
