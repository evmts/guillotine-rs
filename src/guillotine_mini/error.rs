//! Error types for the guillotine-mini REVM adapter
//!
//! # Error Handling Overview
//!
//! This module defines the error types used by the guillotine-mini REVM adapter.
//! Errors are categorized into two main types:
//!
//! ## Database Errors (`EvmAdapterError::Db`)
//!
//! These errors originate from REVM's database layer when loading account state,
//! storage values, or code. The generic `DbErr` type parameter allows this adapter
//! to work with any database implementation that satisfies REVM's `Database` trait.
//!
//! **When it occurs**:
//! - During pre-state synchronization in `database_bridge::sync_account_to_ffi`
//! - When loading contract code in `transact()` method
//! - When reading storage slots via `sync_storage_to_ffi`
//!
//! **Example**:
//! ```rust,no_run
//! use guillotine_rs::guillotine_mini::{GuillotineMiniEvm, EvmAdapterError};
//! use revm::{Context, primitives::{address, TxEnv, TxKind}};
//!
//! let ctx = Context::mainnet();
//! let mut evm = GuillotineMiniEvm::new(ctx);
//!
//! let tx = TxEnv::builder()
//!     .caller(address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"))
//!     .kind(TxKind::Call(address!("0000000000000000000000000000000000000001")))
//!     .gas_limit(100_000)
//!     .build()
//!     .unwrap();
//!
//! match evm.transact(tx) {
//!     Ok(result) => println!("Success: {:?}", result),
//!     Err(EvmAdapterError::Db(e)) => {
//!         eprintln!("Database error: {:?}", e);
//!         // Handle database failure (e.g., retry, use fallback)
//!     }
//!     Err(EvmAdapterError::Ffi(name)) => {
//!         eprintln!("FFI call '{}' failed", name);
//!         // Handle FFI failure (e.g., log, abort)
//!     }
//! }
//! ```
//!
//! ## FFI Errors (`EvmAdapterError::Ffi`)
//!
//! These errors occur when an FFI call to the underlying Zig implementation fails.
//! This typically happens when:
//!
//! - A function returns `false` to indicate failure
//! - A handle creation returns `null`
//! - Invalid parameters are passed (caught at FFI boundary)
//!
//! **When it occurs**:
//! - `evm_create` returns null (EVM instance creation failed)
//! - `evm_set_bytecode` returns false (bytecode too large or invalid)
//! - `evm_set_execution_context` returns false (invalid parameters)
//!
//! The error contains the name of the FFI function that failed, making it easy to
//! identify the source of the problem.
//!
//! **Example**:
//! ```rust,no_run
//! use guillotine_rs::guillotine_mini::{GuillotineMiniEvm, EvmAdapterError};
//! use revm::Context;
//!
//! let ctx = Context::mainnet();
//!
//! match GuillotineMiniEvm::try_new(ctx) {
//!     Ok(evm) => println!("EVM created successfully"),
//!     Err(EvmAdapterError::Ffi("evm_create")) => {
//!         eprintln!("Failed to create EVM instance");
//!         // This is a fatal error - cannot proceed
//!     }
//!     Err(e) => eprintln!("Other error: {:?}", e),
//! }
//! ```
//!
//! ## Error Recovery
//!
//! - **Database errors**: Recoverable - can retry or use alternate database
//! - **FFI errors**: Generally unrecoverable - indicate fundamental initialization failure
//! - **Catastrophic Zig errors**: Cause process abort (panic/unreachable in Zig)
//!
//! Note: Normal EVM execution failures (reverts, out of gas) do NOT produce errors.
//! They are returned as `ExecutionResult::Revert` or similar success variants.

#[derive(Debug)]
pub enum EvmAdapterError<DbErr> {
    /// Database-related error from REVM
    ///
    /// Occurs when loading account state, storage values, or code from the database.
    /// This is a recoverable error that may allow retry or fallback strategies.
    Db(DbErr),

    /// FFI call failed (bool=false or null handle)
    ///
    /// Contains the name of the FFI function that failed. This typically indicates
    /// a fundamental initialization failure or invalid parameters at the FFI boundary.
    Ffi(&'static str),
}

// Conditional Clone implementation when DbErr implements Clone
impl<DbErr: Clone> Clone for EvmAdapterError<DbErr> {
    fn clone(&self) -> Self {
        match self {
            Self::Db(e) => Self::Db(e.clone()),
            Self::Ffi(name) => Self::Ffi(name),
        }
    }
}

// Conditional PartialEq implementation when DbErr implements PartialEq
impl<DbErr: PartialEq> PartialEq for EvmAdapterError<DbErr> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Db(a), Self::Db(b)) => a == b,
            (Self::Ffi(a), Self::Ffi(b)) => a == b,
            _ => false,
        }
    }
}

impl<DbErr: core::fmt::Debug> core::fmt::Display for EvmAdapterError<DbErr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "database error: {:?}", e),
            Self::Ffi(name) => write!(f, "ffi call failed: {}", name),
        }
    }
}

impl<DbErr: core::fmt::Debug> std::error::Error for EvmAdapterError<DbErr> {}

