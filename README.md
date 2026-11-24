# Guillotine-rs

[![CI](https://github.com/evmts/guillotine-rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/evmts/guillotine-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/guillotine-rs)](https://crates.io/crates/guillotine-rs)

Execute EVM calls with [guillotine-mini](https://github.com/evmts/guillotine-mini)
using REVM 30's context, database, transaction, and result types.

## Build

Install Rust 1.91 or newer, [Zig 0.15.2](https://ziglang.org/download/), and a
working native C linker and SDK. Place Zig on `PATH` or set `ZIG` to its absolute
path. CI exercises GNU/Linux x86_64 and macOS AArch64. The build also accepts
GNU/Linux AArch64 and macOS x86_64.

```sh
cargo build --locked
```

The checkout and crate package carry the native source. Once Cargo has fetched
the Rust dependencies, the native build works offline. The [publishing guide](PUBLISHING.md) describes verification for the 0.1.4 release
candidate.

## Execute a contract call

```rust
use guillotine_rs::GuillotineMiniEvm;
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{address, hardfork::SpecId, Bytes, TxKind},
    state::{AccountInfo, Bytecode},
    Context, MainContext,
};

let contract = address!("1000000000000000000000000000000000000000");
// Store 42 in memory and return the 32-byte word.
let code = Bytes::from_static(&[0x60, 42, 0x5f, 0x52, 0x60, 0x20, 0x5f, 0xf3]);
let mut db = InMemoryDB::default();
db.insert_account_info(contract, AccountInfo::default().with_code(Bytecode::new_raw(code)));
let ctx = Context::mainnet()
    .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
    .with_db(db);
let mut evm = GuillotineMiniEvm::try_new(ctx).unwrap();
let tx = TxEnv::builder()
    .kind(TxKind::Call(contract))
    .gas_limit(100_000)
    .build()
    .unwrap();

let outcome = evm.transact(tx).unwrap();
assert!(outcome.result.is_success());
assert_eq!(outcome.result.output().unwrap()[31], 42);
```

The adapter returns output, execution gas, logs and changed storage slots. It
reads the caller and destination from the REVM database and preloads destination
slots 0 through 9. Use REVM itself for full transaction validation, fee and nonce
accounting, contract creation, and complete state access. Access lists, blob
hashes, and custom native configuration are tracked in [#2](https://github.com/evmts/guillotine-rs/issues/2).

## Verify

```sh
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
PROPTEST_CASES=1024 cargo test --locked
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
scripts/test-package.sh
cargo bench --locked --bench transaction
```

The package test extracts the actual `.crate` archive and runs Keccak, BLS and
KZG calls from an independent Cargo consumer. The benchmark reports time per
adapter construction and simple ADD call. [Native source provenance](vendor/README.md)
records the upstream revisions and local changes.
