# Native source provenance

- `guillotine-mini/` comes from the native source tree at
  [fcff7c3](https://github.com/evmts/guillotine-mini/tree/fcff7c3cdbb320f3e8ddc26e2787a4e51163ed3f),
  the revision previously pinned by the submodule. The adapter carries changes
  to initialize the arena in place, preserve pre-state, free FFI buffers, reset
  native state between Rust calls, and report storage writes and execution
  status. The in-place initialization follows the maintained upstream implementation.
- `primitives/` contains the Zig primitives, crypto and precompile modules and
  the curve wrappers from [Voltaire
  9fedbe3](https://github.com/evmts/voltaire/tree/9fedbe3e45e098fd0d9f47d3c336e9b2a4c5f402).
  The Keccak C ABI implementation lives in `src/native_keccak.rs`. The KZG
  precompile verifies the SHA-256 versioned hash and initializes the trusted
  setup on first use.
- `c-kzg/` contains the Zig bindings and trusted setup from the same Voltaire
  revision, plus headers from [c-kzg
  2.1.8](https://crates.io/crates/c-kzg/2.1.8). Keep the headers in sync with the
  exact Cargo dependency. The package compiles the C implementation through
  Cargo's c-kzg and blst dependencies.

The sources are compiled for the Cargo target through `build.zig`. See the
license files in each directory.
