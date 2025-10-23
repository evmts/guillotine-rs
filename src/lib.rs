//! Guillotine-rs: REVM-compatible EVM using Guillotine interpreters
//!
//! This crate provides drop-in replacements for REVM's interpreter using
//! high-performance Zig implementations from the Guillotine project.

pub mod guillotine_mini;

// Re-export for convenience
pub use guillotine_mini::GuillotineMiniInterpreter;
