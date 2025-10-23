//! Guillotine-rs: REVM-compatible EVM using Guillotine interpreters
//!
//! This crate provides REVM integration for high-performance Zig implementations
//! from the Guillotine project.

pub mod guillotine_mini;

// Re-export for convenience
pub use guillotine_mini::GuillotineMiniEvm;
