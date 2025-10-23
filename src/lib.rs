//! Guillotine-rs: REVM-compatible EVM using Guillotine interpreters
//!
//! This crate provides drop-in replacements for REVM's interpreter using
//! high-performance Zig implementations from the Guillotine project.
//!
//! Current status: Building infrastructure, interpreter integration in progress.

pub mod guillotine_mini;

// Re-export for convenience (TODO: uncomment when implementation is ready)
// pub use guillotine_mini::GuillotineMiniInterpreter;
