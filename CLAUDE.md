# CLAUDE.md - Guillotine-rs Development Guide

## Project Overview

**Guillotine-rs** is a high-performance Rust wrapper around the Zig-based [guillotine-mini](https://github.com/evmts/guillotine-mini) EVM engine, providing REVM-compatible transaction execution through FFI. This project bridges Rust's REVM ecosystem with Zig's low-level EVM implementation for optimal performance.

## Architecture

### Core Components

1. **Zig Engine** (`lib/guillotine-mini/`)
   - Minimal EVM implementation in Zig
   - Opcode handlers, storage management, memory management
   - Exposed via C ABI through `root_c.zig`

2. **FFI Layer** (`src/guillotine_mini/ffi.rs`)
   - Raw FFI bindings to Zig's C ABI
   - Lifecycle management (create/destroy EVM instances)
   - State management (storage, balance, nonce, code)
   - Execution context setup and result extraction

3. **REVM Adapter** (`src/guillotine_mini/evm.rs`)
   - High-level wrapper implementing REVM-compatible interface
   - `GuillotineMiniEvm<CTX>` - main execution struct
   - Manages context synchronization between REVM and guillotine-mini
   - Converts execution results to REVM types

4. **Database Bridge** (`src/guillotine_mini/database_bridge.rs`)
   - Syncs pre-state from REVM's `Database` trait to guillotine-mini
   - Handles account info, storage slots, and code
   - Provides error handling for database operations

5. **Configuration API** (`src/guillotine_mini/config.rs`)
   - Type-safe builder for EVM configuration
   - Custom opcode and precompile registration
   - Runtime parameter tuning (stack size, memory limits, etc.)

## Key Files

### Source Code
- `src/lib.rs` - Library root, re-exports main types
- `src/guillotine_mini/mod.rs` - Module organization
- `src/guillotine_mini/evm.rs` - Core EVM wrapper (346 lines)
- `src/guillotine_mini/ffi.rs` - FFI bindings to Zig
- `src/guillotine_mini/error.rs` - Error types
- `src/guillotine_mini/types.rs` - Type conversion utilities
- `src/guillotine_mini/database_bridge.rs` - REVM ↔ Zig state sync
- `src/guillotine_mini/config.rs` - Configuration builder (396 lines)

### Tests
- `tests/revm_compat.rs` - REVM compatibility tests with Ethereum fixtures
- `tests/minimal_test.rs` - Basic FFI tests
- `tests/wrapper_test.rs` - Integration tests
- `tests/config_test.rs` - Configuration API tests

### Build
- `Cargo.toml` - Rust package manifest
- `build.zig` - Zig build script for guillotine-mini
- `lib/guillotine-mini/` - Git submodule with Zig implementation

## Development Workflow

### Building

```bash
# Clone with submodules
git clone https://github.com/evmts/guillotine-rs.git
cd guillotine-rs
git submodule update --init --recursive

# Build Rust crate (automatically builds Zig via build script)
cargo build --release

# Run tests
cargo test

# Run specific test
cargo test test_simple_add -- --nocapture
```

### Testing Strategy

1. **Unit Tests** - In module files (`#[cfg(test)]`)
2. **Integration Tests** - In `tests/` directory
3. **Fixture Tests** - Using Ethereum execution-specs fixtures
4. **FFI Safety Tests** - Verify proper memory management and lifecycle

### Common Tasks

#### Adding a New Opcode Override

```rust
use guillotine_rs::guillotine_mini::{EvmConfigBuilder, GuillotineMiniEvm};

let config = EvmConfigBuilder::new()
    .override_opcode(0x01, |_frame_ptr, _opcode| {
        println!("Custom ADD handler");
        true // Handled
    })
    .build();

let evm = GuillotineMiniEvm::with_config(ctx, config)?;
```

#### Adding a Custom Precompile

```rust
use guillotine_rs::guillotine_mini::{EvmConfigBuilder, PrecompileResult};

let config = EvmConfigBuilder::new()
    .override_precompile(
        [0u8; 20], // Address
        |_addr, input, _gas| {
            Ok(PrecompileResult {
                output: input.to_vec(), // Echo
                gas_used: 100,
            })
        }
    )
    .build();
```

#### Executing a Transaction

```rust
use guillotine_rs::GuillotineMiniEvm;
use revm::{Context, primitives::{address, TxEnv, TxKind}};

let ctx = Context::mainnet().with_db(db);
let mut evm = GuillotineMiniEvm::new(ctx);

let tx = TxEnv::builder()
    .caller(address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"))
    .kind(TxKind::Call(contract_addr))
    .gas_limit(100_000)
    .build()
    .unwrap();

let result = evm.transact(tx)?;
```

## Error Handling

### Error Types

- `EvmAdapterError::Db(DbErr)` - Database errors from REVM
- `EvmAdapterError::Ffi(&'static str)` - FFI call failures

### Constructors

- `GuillotineMiniEvm::new(ctx)` - Panics on FFI failure (convenience)
- `GuillotineMiniEvm::try_new(ctx)` - Returns `Result<Self, EvmAdapterError>`
- `GuillotineMiniEvm::with_config(ctx, config)` - Fallible with custom config

### Execution Results

- `ExecutionResult::Success` - Successful execution with logs, gas, output
- `ExecutionResult::Revert` - Revert with gas used and output
- Zig panics/unreachable → process abort (by design)

## State Management

### Pre-State Synchronization

Before execution, REVM state is synced to guillotine-mini:

1. Caller account (balance, nonce, code)
2. Contract account (balance, nonce, code)
3. Storage slots (on-demand via `sync_storage_to_ffi`)

### Post-State Extraction

After execution, state changes are extracted:

1. Storage changes (grouped by address/slot) via `evm_get_storage_change`
2. Logs (LOG0-LOG4) via `evm_get_log`
3. Gas refunds via `evm_get_gas_refund`

## Hardfork Support

Hardforks map from REVM's `SpecId` to guillotine-mini names:

- Frontier, Homestead, Tangerine, Spurious
- Byzantium, Constantinople, Istanbul
- Berlin, London, Merge
- Shanghai, Cancun, Prague, Osaka

Default: Cancun

## Memory Management

### Ownership Rules

- **FFI Handle**: Created by `evm_create`, destroyed by `evm_destroy` in `Drop`
- **Config Handle**: Created by `evm_config_create`, consumed by `evm_create_with_config`
- **Closures**: Boxed and kept alive in `_opcode_handlers`/`_precompile_handlers` vectors
- **Buffers**: Allocated on Rust side, passed to Zig, never freed by Zig

### Safety Considerations

1. All FFI calls are `unsafe` but wrapped in safe Rust APIs
2. Null pointer checks on all FFI returns
3. Slice bounds verified before FFI calls
4. No data races (enforced by `Send`/`Sync` impls)

## Git Status (Initial)

### Modified Files
- `lib/guillotine-mini` (submodule)
- `src/guillotine_mini/evm.rs`
- `src/guillotine_mini/ffi.rs`
- `src/guillotine_mini/mod.rs`

### New Files
- `.claude/` (this directory)
- `src/guillotine_mini/config.rs`
- `tests/config_test.rs`

### Recent Commits
- `48daf5a` - vibe warning
- `b0bbbca` - Change logo image in README.md
- `0246a0b` - test: add revert execution result mapping test ✅
- `f4d57e0` - docs: rewrite README with comprehensive API documentation 📚
- `56768f3` - docs: add comprehensive README for project 📚

## Dependencies

### Rust
- `revm = "^30.2.0"` - Ethereum Virtual Machine
- `alloy = "^1.0.41"` - Ethereum primitives
- `hex = "0.4.3"` (dev) - Hex encoding/decoding

### System
- Rust 1.75+
- Zig 0.15.1+
- Git (for submodules)

## Performance Characteristics

- **Execution**: Zig-native, highly optimized opcode dispatch
- **State Sync**: Minimal overhead, only syncs accessed accounts
- **Storage**: Lazy loading, only extracts non-zero slots post-execution
- **Logs**: Zero-copy when possible, heap allocation for variable-length data

## Known Limitations

1. Storage extraction only returns non-zero slots (by design)
2. Catastrophic Zig errors cause process abort (cannot recover)
3. Precompile output allocation uses `std::mem::forget` (intentional leak to C)
4. No support for CREATE2 nonce handling (inherits REVM behavior)

## Testing Coverage

- ✅ Basic arithmetic operations
- ✅ Storage writes across multiple slots
- ✅ Gas refund behavior (SSTORE operations)
- ✅ Log emission (LOG0-LOG4)
- ✅ Revert handling with proper result mapping
- ✅ Hardfork-specific opcodes (CHAINID, etc.)
- ✅ Configuration API (builder pattern)
- ⏳ EIP-4844 blob transactions (partial)
- ⏳ System contracts (beacon roots, deposits, etc.)
- ⏳ Custom precompile stress tests

## API Stability

- **Stable**: `GuillotineMiniEvm::new`, `transact`, FFI lifecycle
- **Unstable**: Configuration API (may change), precompile interface

## Contributing

See `lib/guillotine-mini/CONTRIBUTING.md` for Zig-side contribution guidelines.

For Rust-side changes:

1. Run `cargo fmt` before committing
2. Ensure `cargo clippy` passes
3. Add tests for new functionality
4. Update README.md API documentation if public API changes
5. Use conventional commit format: `feat:`, `fix:`, `test:`, `docs:`, `refactor:`

## Resources

- [Guillotine Mini](https://github.com/evmts/guillotine-mini) - Zig EVM implementation
- [REVM](https://github.com/bluealloy/revm) - Rust Ethereum Virtual Machine
- [Ethereum Execution Specs](https://github.com/ethereum/execution-specs) - Test fixtures
- [EVM Opcodes](https://www.evm.codes/) - Reference documentation
