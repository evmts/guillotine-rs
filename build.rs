//! Build script to compile guillotine-mini Zig library

use std::process::Command;
use std::path::PathBuf;
use std::env;

/// Check if a command exists in PATH (cross-platform)
fn command_exists(cmd: &str) -> bool {
    // For zig specifically, use 'version' without dashes
    // This is more reliable than 'which' on Windows
    Command::new(cmd)
        .arg("version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get zig version if installed
fn get_zig_version() -> Option<String> {
    Command::new("zig")
        .arg("version")
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
}

/// Check if zig version meets minimum requirement (0.15.1)
fn check_zig_version(version: &str) -> bool {
    // Parse version string (e.g., "0.15.1" -> [0, 15, 1])
    let parts: Vec<u32> = version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect();

    if parts.len() < 3 {
        return false;
    }

    // Check against minimum version 0.15.1
    parts[0] > 0 || (parts[0] == 0 && parts[1] > 15) || (parts[0] == 0 && parts[1] == 15 && parts[2] >= 1)
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR environment variable not set"));

    // Note: Not monitoring our build.zig/build.zig.zon since we use guillotine-mini's build system
    println!("cargo:rerun-if-changed=lib/guillotine-mini/src");
    println!("cargo:rerun-if-changed=lib/guillotine-mini/build.zig");

    // Check if guillotine-mini submodule is initialized
    let submodule_src = manifest_dir.join("lib/guillotine-mini/src");
    if !submodule_src.exists() {
        eprintln!("\n========================================");
        eprintln!("ERROR: guillotine-mini submodule not initialized");
        eprintln!("========================================");
        eprintln!("\nThe guillotine-mini submodule has not been initialized.\n");
        eprintln!("Please run the following commands:");
        eprintln!("  git submodule update --init --recursive");
        eprintln!("========================================\n");
        panic!("guillotine-mini submodule not initialized");
    }

    // Check if Zig is installed
    if !command_exists("zig") {
        eprintln!("\n========================================");
        eprintln!("ERROR: Zig compiler not found!");
        eprintln!("========================================");
        eprintln!("\nguillotine-rs requires Zig 0.15.1 or later to build.\n");
        eprintln!("Please install Zig:");
        eprintln!("  - Download: https://ziglang.org/download/");
        eprintln!("  - macOS:    brew install zig");
        eprintln!("  - Linux:    See https://ziglang.org/download/");
        eprintln!("  - Windows:  See https://ziglang.org/download/\n");
        eprintln!("After installation, verify with: zig version");
        eprintln!("========================================\n");
        panic!("Zig compiler not found in PATH");
    }

    // Check Zig version
    match get_zig_version() {
        Some(version) => {
            eprintln!("Found Zig version: {}", version);
            if !check_zig_version(&version) {
                eprintln!("\n========================================");
                eprintln!("ERROR: Zig version too old!");
                eprintln!("========================================");
                eprintln!("\nFound Zig {}, but guillotine-rs requires Zig 0.15.1 or later.\n", version);
                eprintln!("Please upgrade Zig:");
                eprintln!("  - Download: https://ziglang.org/download/");
                eprintln!("  - macOS:    brew upgrade zig");
                eprintln!("========================================\n");
                panic!("Zig version {} is too old (need 0.15.1+)", version);
            }
        }
        None => {
            eprintln!("WARNING: Could not determine Zig version, proceeding anyway...");
        }
    }

    // Build guillotine-mini using zig build-deps (just Zig, not cargo)
    eprintln!("Building guillotine-mini Zig library from submodule...");

    // Use OUT_DIR for zig build artifacts to keep source tree clean
    let out_dir = PathBuf::from(env::var("OUT_DIR")
        .expect("OUT_DIR environment variable not set"));
    let zig_cache_dir = out_dir.join(".zig-cache");
    let zig_out_dir = out_dir.join("zig-out");

    // Build guillotine-mini using its native target (for FFI)
    // This automatically handles primitives dependency fetching and Rust component building
    eprintln!("Building guillotine-mini native library...");
    let guillotine_mini_dir = manifest_dir.join("lib/guillotine-mini");

    let status = Command::new("zig")
        .args(&[
            "build",
            "native",  // Use native target for FFI integration
            "--prefix", zig_out_dir.to_str()
                .expect("Failed to convert zig output directory path to string"),
        ])
        .current_dir(&guillotine_mini_dir)
        .status()
        .expect("Failed to execute zig build command");

    if !status.success() {
        eprintln!("\n========================================");
        eprintln!("ERROR: Zig build failed");
        eprintln!("========================================");
        eprintln!("\nThe Zig compiler encountered an error while building guillotine-mini.");
        eprintln!("\nPlease report this issue at:");
        eprintln!("  https://github.com/evmts/guillotine-rs/issues");
        eprintln!("========================================\n");
        panic!("zig build failed");
    }

    // Validate that the build artifacts were created successfully
    let lib_path = zig_out_dir.join("lib/libguillotine_mini.a");
    if !lib_path.exists() {
        eprintln!("\n========================================");
        eprintln!("ERROR: Build artifact not found");
        eprintln!("========================================");
        eprintln!("\nExpected library not found at: {:?}", lib_path);
        eprintln!("The Zig build may have completed but failed to produce the library.");
        eprintln!("\nPlease report this issue at:");
        eprintln!("  https://github.com/evmts/guillotine-rs/issues");
        eprintln!("========================================\n");
        panic!("Expected library not found at {:?}. Build may have failed.", lib_path);
    }

    let lib_metadata = std::fs::metadata(&lib_path)
        .expect("Failed to read metadata for libguillotine_mini.a");
    if lib_metadata.len() == 0 {
        eprintln!("\n========================================");
        eprintln!("ERROR: Build artifact is empty");
        eprintln!("========================================");
        eprintln!("\nLibrary at {:?} exists but is empty (0 bytes).", lib_path);
        eprintln!("The Zig build may have failed silently.");
        eprintln!("\nPlease report this issue at:");
        eprintln!("  https://github.com/evmts/guillotine-rs/issues");
        eprintln!("========================================\n");
        panic!("Library at {:?} is empty. Build may have failed.", lib_path);
    }

    // Tell cargo where to find the libraries
    let lib_dir = zig_out_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=guillotine_mini");

    // Also link primitives_c from the zig cache
    let zig_cache_lib_dir = zig_cache_dir.join("o");
    let mut primitives_found = false;

    // Find primitives_c in cache subdirectories
    if let Ok(entries) = std::fs::read_dir(&zig_cache_lib_dir) {
        for entry in entries.flatten() {
            let primitives_lib = entry.path().join("libprimitives_c.a");
            if primitives_lib.exists() {
                println!("cargo:rustc-link-search=native={}", entry.path().display());
                println!("cargo:rustc-link-lib=static=primitives_c");
                eprintln!("Found primitives_c: {}", primitives_lib.display());
                primitives_found = true;
                break;
            }
        }
    }

    if !primitives_found {
        eprintln!("\n========================================");
        eprintln!("WARNING: primitives_c library not found");
        eprintln!("========================================");
        eprintln!("\nCould not locate libprimitives_c.a in Zig cache directory:");
        eprintln!("  {:?}", zig_cache_lib_dir);
        eprintln!("\nThis may cause linking errors. If the build fails, please:");
        eprintln!("  1. Clean the build: cargo clean");
        eprintln!("  2. Rebuild: cargo build");
        eprintln!("  3. If the issue persists, report at:");
        eprintln!("     https://github.com/evmts/guillotine-rs/issues");
        eprintln!("========================================\n");
    }

    eprintln!("guillotine-mini native library built: {}/libguillotine_mini.a",
              lib_dir.display());
}
