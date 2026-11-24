//! Compile the vendored Zig EVM and link it into the Rust crate.

use std::{env, path::PathBuf, process::Command};

fn main() {
    for path in ["build.zig", "vendor"] {
        println!("cargo:rerun-if-changed={path}");
    }
    println!("cargo:rerun-if-env-changed=ZIG");

    // docs.rs generates API documentation without linking native binaries.
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let zig = env::var_os("ZIG").unwrap_or_else(|| "zig".into());
    let version = Command::new(&zig)
        .arg("version")
        .output()
        .expect("Zig 0.15.2 is required. Install it or set ZIG to the compiler path");
    let installed = String::from_utf8_lossy(&version.stdout);
    assert!(
        version.status.success() && installed.trim() == "0.15.2",
        "expected Zig 0.15.2, found {:?}. Set ZIG to the supported compiler",
        installed.trim()
    );

    let target = env::var("TARGET").unwrap();
    let zig_target = match target.as_str() {
        "aarch64-apple-darwin" => "aarch64-macos",
        "x86_64-apple-darwin" => "x86_64-macos",
        "aarch64-unknown-linux-gnu" => "aarch64-linux-gnu",
        "x86_64-unknown-linux-gnu" => "x86_64-linux-gnu",
        _ => panic!(
            "unsupported target {target}. Use a GNU/Linux or macOS target on x86_64 or AArch64"
        ),
    };

    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let status = Command::new(&zig)
        .args(["build", "-Doptimize=ReleaseSafe"])
        .arg(format!("-Dtarget={zig_target}"))
        .arg("--cache-dir")
        .arg(out.join("zig-cache"))
        .arg("--prefix")
        .arg(out.join("zig-out"))
        .status()
        .expect("failed to run zig build");
    assert!(status.success(), "failed to build the Zig EVM: {status}");
    let lib = out.join("zig-out/lib");
    assert!(
        lib.join("libguillotine_mini.a").is_file(),
        "Zig did not install the EVM archive"
    );
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=static=guillotine_mini");
}
