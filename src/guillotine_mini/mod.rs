//! Guillotine-mini REVM adapter
//!
//! This module provides a REVM-compatible EVM backed by guillotine-mini's
//! Zig implementation via native FFI.
//!
//! # Configuration API Status
//!
//! The configuration API (`config` module) is temporarily disabled pending upstream FFI support
//! in guillotine-mini. The config module provides:
//!
//! - Custom opcode handlers via `EvmConfigBuilder::override_opcode`
//! - Custom precompile registration via `EvmConfigBuilder::override_precompile`
//! - Runtime parameter tuning (stack size, memory limits, gas limits, etc.)
//! - System contract feature flags
//!
//! **Current Status**: The Rust-side configuration API is implemented and tested, but the
//! corresponding FFI functions in guillotine-mini (commit: 25b2185) are not yet available in
//! the stable C ABI. Once upstream adds these functions to `root_c.zig`, the config module
//! will be re-enabled.
//!
//! **Tracking**: See commit 25b2185 - "refactor: Temporarily disable config API pending upstream FFI"
//!
//! **Workaround**: Use the default EVM configuration via `GuillotineMiniEvm::new()` or
//! `GuillotineMiniEvm::try_new()`. These constructors create an EVM instance with standard
//! hardfork-based configuration.

// TODO: Re-enable once guillotine-mini upstream adds config FFI functions
// The config API is fully implemented but requires upstream FFI support:
// - evm_config_create()
// - evm_config_set_* functions
// - evm_config_add_opcode_override()
// - evm_config_add_precompile_override()
// - evm_create_with_config()
// pub mod config;
pub mod database_bridge;
pub mod evm;
pub mod ffi;
pub mod error;
pub mod types;

pub use evm::GuillotineMiniEvm;
pub use error::EvmAdapterError;
pub use database_bridge::{sync_account_to_ffi, sync_storage_to_ffi, sync_storage_slots_to_ffi};
// TODO: Re-enable once guillotine-mini upstream adds config FFI functions
// pub use config::{EvmConfigBuilder, EvmConfig, PrecompileResult, PrecompileError};
