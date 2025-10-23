Guillotine-RS: REVM integration for guillotine-mini (Zig)

Overview
- High-performance REVM execution backed by the Zig-based guillotine-mini engine.
- Thin Rust wrapper with FFI to Zig for execution, state sync, logs, refunds, and storage changes.

Key Features
- REVM-compatible transaction execution with guillotine-mini.
- Pre-state sync: balances, nonces, code, and storage.
- Post-state extraction: storage changes grouped by address/slot.
- Gas refund exposure: direct from guillotine-mini’s runtime counter.
- Log emission: LOG0–LOG4 captured in Zig and returned to Rust as REVM logs.

Architecture
- Zig (lib/guillotine-mini): core EVM, opcode handlers, storage manager.
- FFI layer (root_c.zig): stable C ABI to create/destroy EVM, set contexts, execute, and extract results.
- Rust wrapper (src/guillotine_mini): REVM adapter, type conversions, and state bridge.

Usage
- Build and run tests: `cargo test`
- Use `GuillotineMiniEvm` with a REVM `Context` to execute a `TxEnv`.

FFI Additions
- Logs:
  - `evm_get_log_count(handle) -> usize`
  - `evm_get_log(handle, index, address_out, topics_count_out, topics_out, data_len_out, data_out, data_max_len) -> bool`
- Gas refunds:
  - `evm_get_gas_refund(handle) -> u64`
- Storage changes:
  - `evm_get_storage_change_count(handle) -> usize`
  - `evm_get_storage_change(handle, index, address_out, slot_out, value_out) -> bool`

Notes and Limits
- Storage extraction enumerates final non-zero slots; zeroed slots are not emitted.
- Logs are now emitted by Zig’s LOG handlers and included in results.
- Error handling currently asserts on FFI failures; can be converted to typed errors if needed.

Testing
- Added tests for storage writes across multiple slots, gas refund behavior, and basic log emission.

