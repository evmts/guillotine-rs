//! Guillotine-mini interpreter implementation
//!
//! Implements REVM's InterpreterTypes trait using guillotine-mini as the backend.

use super::ffi::EvmHandle;

/// Guillotine-mini interpreter type
///
/// This will implement REVM's InterpreterTypes trait to provide
/// a drop-in replacement for EthInterpreter.
pub struct GuillotineMiniInterpreter {
    handle: *mut EvmHandle,
}

// TODO: Implement InterpreterTypes trait
// TODO: Implement stack, memory, bytecode wrappers
// TODO: Implement instruction execution
