//! Build script to compile guillotine-mini Zig library

use std::process::Command;
use std::path::PathBuf;
use std::env;

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
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
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.zig");
    println!("cargo:rerun-if-changed=build.zig.zon");

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
    eprintln!("Building guillotine-mini Zig library...");

    // Use OUT_DIR for zig build artifacts to keep source tree clean
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let zig_cache_dir = out_dir.join(".zig-cache");
    let zig_out_dir = out_dir.join("zig-out");

    let status = Command::new("zig")
        .args(&[
            "build",
            "build-deps",  // Use build-deps step to avoid circular cargo build
            "--cache-dir", zig_cache_dir.to_str().unwrap(),
            "--prefix", zig_out_dir.to_str().unwrap(),
        ])
        .current_dir(&manifest_dir)
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

    // Tell cargo where to find the libraries
    let lib_dir = zig_out_dir.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=guillotine_mini");

    // Also link crypto_wrappers from the zig cache
    let zig_cache_lib_dir = zig_cache_dir.join("o");
    // Find crypto_wrappers in cache subdirectories
    if let Ok(entries) = std::fs::read_dir(&zig_cache_lib_dir) {
        for entry in entries.flatten() {
            let crypto_lib = entry.path().join("libcrypto_wrappers.a");
            if crypto_lib.exists() {
                println!("cargo:rustc-link-search=native={}", entry.path().display());
                println!("cargo:rustc-link-lib=static=crypto_wrappers");
                eprintln!("Found crypto_wrappers: {}", crypto_lib.display());
                break;
            }
        }
    }

    eprintln!("guillotine-mini native library built: {}/libguillotine_mini.a",
              lib_dir.display());
}
