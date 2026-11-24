# Publishing guillotine-rs

The crate archive includes the pinned Zig sources, C headers, native build
scripts, and licenses. Cargo resolves the Rust crypto crates, builds the native
dependencies, and links them into downstream binaries. The build needs Zig
0.15.2 and the native target's C toolchain.

1. Run the [CI checks](README.md#verify) on the proposed release.
2. Run `scripts/test-package.sh`. It runs `cargo package --locked --allow-dirty`,
   extracts the produced archive and builds and runs a separate Cargo consumer
   against it. That consumer covers Keccak-256, BLS12-381, and KZG.
3. Run `cargo publish --locked --dry-run` and check the release version against
   [crates.io](https://crates.io/crates/guillotine-rs).
4. After the release is approved, publish the verified version with
   `cargo publish --locked`.
