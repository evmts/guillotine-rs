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
    let guillotine_mini_dir = manifest_dir.join("lib/guillotine-mini");

    println!("cargo:rerun-if-changed=lib/guillotine-mini/src");
    println!("cargo:rerun-if-changed=lib/guillotine-mini/build.zig");

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

    // Check if git is available (needed for submodules)
    if !command_exists("git") {
        eprintln!("\n========================================");
        eprintln!("ERROR: git not found!");
        eprintln!("========================================");
        eprintln!("\nguillotine-rs requires git to initialize submodules.\n");
        eprintln!("Please install git: https://git-scm.com/downloads");
        eprintln!("========================================\n");
        panic!("git not found in PATH");
    }

    // Check if submodules are initialized
    let primitives_dir = guillotine_mini_dir.join("lib/primitives");
    if !primitives_dir.join("src").exists() {
        eprintln!("Initializing guillotine-mini submodules...");
        let status = Command::new("git")
            .args(&["submodule", "update", "--init", "--recursive"])
            .current_dir(&manifest_dir)
            .status()
            .expect("Failed to execute git submodule command");

        if !status.success() {
            eprintln!("\n========================================");
            eprintln!("ERROR: Failed to initialize git submodules");
            eprintln!("========================================");
            eprintln!("\nIf you installed via cargo, try cloning manually:");
            eprintln!("  git clone --recursive https://github.com/evmts/guillotine-rs");
            eprintln!("  cd guillotine-rs");
            eprintln!("  cargo build");
            eprintln!("========================================\n");
            panic!("git submodule init failed");
        }
    }

    // Build guillotine-mini native static library using zig
    eprintln!("Building guillotine-mini Zig library...");
    let status = Command::new("zig")
        .args(&["build", "native"])
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
