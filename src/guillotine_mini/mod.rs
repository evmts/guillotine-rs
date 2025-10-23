//! Guillotine-mini REVM adapter
//!
//! This module provides a REVM-compatible EVM backed by guillotine-mini's
//! Zig implementation via native FFI.

pub mod database_bridge;
pub mod evm;
pub mod ffi;
pub mod types;

// Legacy interpreter stub (deprecated in favor of evm wrapper)
#[deprecated(note = "Use GuillotineMiniEvm instead")]
pub mod interpreter;

pub use evm::GuillotineMiniEvm;
