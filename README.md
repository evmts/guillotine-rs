<div align="center">
  <h1>
    REVM integration for guillotine
    <br/>
    <br/>
    <img width="1024" height="1024" alt="guillotine-rs-logo" src="https://github.com/user-attachments/assets/d8107fbd-12b2-4bb4-be7e-de86c3fced44" />
  </h1>
  <sup>
    <a href="https://github.com/evmts/guillotine-rs">
       <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="rust version" />
    </a>
    <a href="https://github.com/evmts/guillotine-rs/actions">
      <img src="https://img.shields.io/badge/build-passing-brightgreen.svg" alt="build status" />
    </a>
    <a href="https://github.com/evmts/guillotine-rs">
      <img src="https://img.shields.io/badge/tests-passing-brightgreen.svg" alt="tests" />
    </a>
  </sup>
</div>

## Requirements

[Rust 1.75+ (Cargo)](https://www.rust-lang.org/tools/install) — [Zig 0.15.1+](https://ziglang.org/download/)

## Installation

### WARNING: This repo is currently vibes and hasn't been reviewed by a human yet

**Recommended:** Build from source

```bash
git clone https://github.com/evmts/guillotine-rs.git
cd guillotine-rs
git submodule update --init --recursive
cargo build --release
```

**Alternative:** Add to your Cargo.toml

```toml
[dependencies]
guillotine-rs = { git = "https://github.com/evmts/guillotine-rs" }
```

<br />

## Documentation

[`tests/`](./tests/) — Example code and usage

[`LLMS.txt`](./LLMS.txt) — For LLMs

## Overview

High-performance REVM execution backed by the Zig-based [guillotine-mini](https://github.com/evmts/guillotine-mini) engine. Thin Rust wrapper with FFI to Zig for execution, state sync, logs, refunds, and storage changes.

## Architecture

- **Zig** ([`lib/guillotine-mini`](./lib/guillotine-mini)) — core EVM, opcode handlers, storage manager
- **FFI layer** (`root_c.zig` in guillotine-mini) — stable C ABI to create/destroy EVM, set contexts, execute, and extract results
- **Rust wrapper** ([`src/guillotine_mini`](./src/guillotine_mini)) — REVM adapter, type conversions, and state bridge

## Key Features

- **REVM-compatible** — Drop-in transaction execution with REVM's `Context` and `TxEnv`
- **Pre-state sync** — Automatically syncs balances, nonces, code, and storage to guillotine-mini
- **Post-state extraction** — Storage changes grouped by address/slot
- **Gas refunds** — Direct exposure from guillotine-mini's runtime counter
- **Log emission** — LOG0–LOG4 captured in Zig and returned as REVM logs
- **Typed errors** — Proper error handling with `EvmAdapterError<DbErr>`

## API

**Legend**: All FFI calls are wrapped with safe Rust interfaces

- [**REVM Wrapper**](#revm-wrapper)
  - [`GuillotineMiniEvm`](./src/guillotine_mini/evm.rs) — main EVM wrapper for REVM integration
    - [`new`](./src/guillotine_mini/evm.rs#L34) — create EVM instance from REVM context (panics on FFI failure)
    - [`try_new`](./src/guillotine_mini/evm.rs#L68) — fallible constructor returning `Result<Self, EvmAdapterError>`
    - [`transact`](./src/guillotine_mini/evm.rs#L98) — execute transaction and return `ResultAndState`
  - [`EvmAdapterError`](./src/guillotine_mini/error.rs) — typed error handling
    - `Db(DbErr)` — database-related error from REVM
    - `Ffi(&'static str)` — FFI call failed (bool=false or null handle)
      <br/>
      <br/>
- [**Database Bridge**](#database-bridge)
  - [`sync_account_to_ffi`](./src/guillotine_mini/database_bridge.rs#L14) — sync REVM account state to guillotine-mini (balance, nonce, code)
  - [`sync_storage_to_ffi`](./src/guillotine_mini/database_bridge.rs#L58) — sync single storage slot to guillotine-mini
  - [`read_storage_from_ffi`](./src/guillotine_mini/database_bridge.rs#L87) — read storage value from guillotine-mini
    <br/>
    <br/>
- [**FFI Bindings**](#ffi-bindings)
  - **Lifecycle**
    - [`evm_create`](./src/guillotine_mini/ffi.rs#L29) — create EVM instance with hardfork name
    - [`evm_destroy`](./src/guillotine_mini/ffi.rs#L34) — free EVM resources
  - **Configuration**
    - [`evm_set_bytecode`](./src/guillotine_mini/ffi.rs#L42) — set contract bytecode for execution
    - [`evm_set_execution_context`](./src/guillotine_mini/ffi.rs#L52) — set caller, address, value, gas, calldata
    - [`evm_set_block_context`](./src/guillotine_mini/ffi.rs#L67) — set block number, timestamp, gas limit, etc.
  - **Execution**
    - [`evm_execute`](./src/guillotine_mini/ffi.rs#L82) — execute transaction and return success/failure
    - [`evm_get_status`](./src/guillotine_mini/ffi.rs#L86) — check if execution succeeded
    - [`evm_get_gas_used`](./src/guillotine_mini/ffi.rs#L91) — get gas consumed by execution
    - [`evm_get_gas_refund`](./src/guillotine_mini/ffi.rs#L212) — get gas refund counter
  - **Output**
    - [`evm_get_output_size`](./src/guillotine_mini/ffi.rs#L96) — get return data length
    - [`evm_copy_output`](./src/guillotine_mini/ffi.rs#L101) — copy return data to buffer
  - **State Management**
    - [`evm_set_storage`](./src/guillotine_mini/ffi.rs#L128) — set storage slot value
    - [`evm_get_storage`](./src/guillotine_mini/ffi.rs#L141) — get storage slot value
    - [`evm_set_balance`](./src/guillotine_mini/ffi.rs#L153) — set account balance
    - [`evm_set_code`](./src/guillotine_mini/ffi.rs#L165) — set account code
    - [`evm_set_nonce`](./src/guillotine_mini/ffi.rs#L180) — set account nonce
  - **Result Introspection**
    - [`evm_get_log_count`](./src/guillotine_mini/ffi.rs#L192) — get number of emitted logs
    - [`evm_get_log`](./src/guillotine_mini/ffi.rs#L203) — get log entry by index (address, topics, data)
    - [`evm_get_storage_change_count`](./src/guillotine_mini/ffi.rs#L215) — get number of storage changes
    - [`evm_get_storage_change`](./src/guillotine_mini/ffi.rs#L224) — get storage change by index (address, slot, value)
      <br/>
      <br/>
- [**Type Conversions**](#type-conversions)
  - [`address_to_bytes`](./src/guillotine_mini/types.rs#L9) — convert REVM Address to [20]u8
  - [`address_from_bytes`](./src/guillotine_mini/types.rs#L14) — convert [20]u8 to REVM Address
  - [`u256_to_be_bytes`](./src/guillotine_mini/types.rs#L24) — convert U256 to big-endian [32]u8
  - [`u256_from_be_bytes`](./src/guillotine_mini/types.rs#L34) — convert big-endian [32]u8 to U256
  - [`i64_to_u64_gas`](./src/guillotine_mini/types.rs#L48) — convert signed gas to unsigned (clamp negative)

## Error Handling

- **Reverts** — Mapped to `ExecutionResult::Revert { gas_used, output }` (no panic)
- **Success** — Returns `ExecutionResult::Success { reason: Return, gas_used, gas_refunded, logs, output }`
- **FFI failures** — Properly propagated via `EvmAdapterError::Ffi(&'static str)`
- **Database errors** — Wrapped in `EvmAdapterError::Db(DbErr)` and propagated
- **Catastrophic failures** — Zig panic/unreachable causes process abort (by design)

Fallible constructor available: `GuillotineMiniEvm::try_new(ctx)` returns `Result<Self, EvmAdapterError>`. The `new(ctx)` constructor retains an assert on fatal creation failure for convenience.

## Usage

```rust
use guillotine_rs::guillotine_mini::evm::GuillotineMiniEvm;
use revm::{
    context::Context,
    context_interface::result::ExecutionResult,
    database_interface::EmptyDB,
    primitives::{address, TxEnv, TxKind, U256},
};

// Create REVM context
let ctx = Context::mainnet().with_db(EmptyDB::default());
let mut evm = GuillotineMiniEvm::new(ctx);

// Build transaction
let tx = TxEnv::builder()
    .caller(address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"))
    .kind(TxKind::Call(address!("0000000000000000000000000000000000000001")))
    .gas_limit(100_000)
    .build()
    .unwrap();

// Execute and get results
let result = evm.transact(tx).unwrap();
match result.result {
    ExecutionResult::Success { gas_used, gas_refunded, logs, output, .. } => {
        println!("Success! Gas used: {}, refunded: {}", gas_used, gas_refunded);
        println!("Logs: {}, Output: {:?}", logs.len(), output);
    }
    ExecutionResult::Revert { gas_used, output } => {
        println!("Reverted! Gas used: {}, Output: {:?}", gas_used, output);
    }
    _ => unreachable!(),
}
```

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_simple_add

# Run with output
cargo test -- --nocapture
```

**Test coverage:**

- [Storage writes across multiple slots](./tests/revm_compat.rs#L186)
- [Gas refund behavior on SSTORE operations](./tests/revm_compat.rs#L228)
- [Log emission (LOG0 instruction)](./tests/revm_compat.rs#L141)
- [Revert handling with proper ExecutionResult mapping](./tests/revm_compat.rs#L186)
- [Basic arithmetic operations](./tests/revm_compat.rs#L115)

## Notes and Limits

- Storage extraction enumerates final non-zero slots; zeroed slots are not emitted
- Logs are emitted by Zig's LOG handlers and included in results
- All hardforks from Frontier to Osaka are supported via REVM's SpecId mapping
- Submodule ([guillotine-mini](https://github.com/evmts/guillotine-mini)) must be initialized: `git submodule update --init --recursive`

## More

[**Guillotine Mini**](https://github.com/evmts/guillotine-mini) — Minimal Zig EVM implementation

[**Primitives**](https://github.com/evmts/primitives) — Ethereum primitives and cryptography for Zig

[**REVM**](https://github.com/bluealloy/revm) — Rust Ethereum Virtual Machine
