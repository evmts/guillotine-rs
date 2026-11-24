//! Execute EVM calls with guillotine-mini using REVM context and database types.
//!
//! The native dependency builds from the source bundled in the crate. Install
//! Zig 0.15.2 and set `ZIG` if it is outside `PATH`.
//!
//! ```
//! use guillotine_rs::GuillotineMiniEvm;
//! use revm::{context::TxEnv, primitives::TxKind, Context, MainContext};
//!
//! let mut evm = GuillotineMiniEvm::try_new(Context::mainnet()).unwrap();
//! let call = TxEnv::builder()
//!     .kind(TxKind::Call(revm::primitives::address!("1000000000000000000000000000000000000000")))
//!     .gas_limit(100_000)
//!     .build()
//!     .unwrap();
//! assert!(evm.transact(call).unwrap().result.is_success());
//! ```

pub mod guillotine_mini;

// Re-export for convenience
pub use guillotine_mini::{EvmAdapterError, GuillotineMiniEvm};

// The Zig archive calls the C symbols supplied by this Cargo dependency.
extern crate c_kzg as _;

// The vendored enums reserve unused return codes for their public C ABI.
#[allow(dead_code, clippy::manual_is_multiple_of)]
#[path = "../vendor/primitives/ffi/bn254_wrapper.rs"]
mod bn254_wrapper;
mod native_keccak;
