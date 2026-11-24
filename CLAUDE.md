# Guillotine-rs development

Use the toolchains and verification commands in [README.md](README.md#verify).
The package regression checks must exercise the archive in a separate consumer.

- `src/guillotine_mini/evm.rs` adapts the REVM context, transaction input, execution
  result and storage changes.
- `src/guillotine_mini/database_bridge.rs` seeds Zig state from the REVM database.
- `src/guillotine_mini/ffi.rs` declares the native C ABI. Keep its layouts and
  signatures synchronized with `vendor/guillotine-mini/root_c.zig`.
- `build.rs` and `build.zig` build the vendored Zig source. Cargo links the
  corresponding Rust and C crypto code directly.
- [Native source provenance](vendor/README.md) records the upstream revisions.
- [Publishing](PUBLISHING.md) documents crate archive verification.
