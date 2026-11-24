//! Guillotine-mini integration with REVM.
//!
//! Start with [`GuillotineMiniEvm`] for database-backed execution. The [`ffi`]
//! and [`database_bridge`] modules expose lower-level functions for callers
//! that manage their own native handle.

pub mod database_bridge;
pub mod error;
pub mod evm;
pub mod ffi;
pub mod types;

pub use database_bridge::{sync_account_to_ffi, sync_storage_slots_to_ffi, sync_storage_to_ffi};
pub use error::EvmAdapterError;
pub use evm::GuillotineMiniEvm;
