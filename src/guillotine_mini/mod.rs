//! Guillotine-mini REVM adapter
//!
//! This module provides a REVM-compatible interpreter backed by guillotine-mini's
//! Zig implementation.

pub mod ffi;
pub mod interpreter;
pub mod types;

pub use interpreter::GuillotineMiniInterpreter;
