//! High-level configuration API for guillotine-mini EVM
//!
//! Provides a safe, type-safe builder for configuring the EVM with custom
//! opcodes, precompiles, and runtime parameters.

use super::ffi;
use std::ffi::c_void;

/// Result type for precompile execution
#[derive(Debug, Clone)]
pub struct PrecompileResult {
    pub output: Vec<u8>,
    pub gas_used: u64,
}

/// Error type for precompile execution
#[derive(Debug, Clone)]
pub enum PrecompileError {
    OutOfGas,
    InvalidInput,
    ExecutionFailed(String),
}

/// Type-safe configuration builder for guillotine-mini EVM
pub struct EvmConfigBuilder {
    handle: *mut ffi::EvmConfigHandle,
    // Keep closures alive for their lifetime
    _opcode_handlers: Vec<Box<OpcodeHandlerFn>>,
    _precompile_handlers: Vec<Box<PrecompileHandlerFn>>,
}

// Boxed trait objects for opcode and precompile handlers
type OpcodeHandlerFn = dyn Fn(usize, u8) -> bool + Send + Sync + 'static;
type PrecompileHandlerFn =
    dyn Fn(&[u8], &[u8], u64) -> Result<PrecompileResult, PrecompileError> + Send + Sync + 'static;

impl EvmConfigBuilder {
    /// Create a new configuration builder with default values
    pub fn new() -> Self {
        let handle = unsafe { ffi::evm_config_create() };
        assert!(!handle.is_null(), "Failed to create EVM config");

        Self {
            handle,
            _opcode_handlers: Vec::new(),
            _precompile_handlers: Vec::new(),
        }
    }

    /// Set the hardfork for EVM execution
    ///
    /// # Example
    /// ```ignore
    /// let config = EvmConfigBuilder::new()
    ///     .hardfork("Cancun")
    ///     .build();
    /// ```
    pub fn hardfork(self, name: &str) -> Self {
        unsafe {
            ffi::evm_config_set_hardfork(self.handle, name.as_ptr(), name.len());
        }
        self
    }

    /// Set maximum stack size (default: 1024)
    pub fn stack_size(self, size: u16) -> Self {
        unsafe {
            ffi::evm_config_set_stack_size(self.handle, size);
        }
        self
    }

    /// Set maximum bytecode size (default: 24576)
    pub fn max_bytecode_size(self, size: u32) -> Self {
        unsafe {
            ffi::evm_config_set_max_bytecode_size(self.handle, size);
        }
        self
    }

    /// Set maximum initcode size (default: 49152)
    pub fn max_initcode_size(self, size: u32) -> Self {
        unsafe {
            ffi::evm_config_set_max_initcode_size(self.handle, size);
        }
        self
    }

    /// Set block gas limit (default: 30000000)
    pub fn block_gas_limit(self, limit: u64) -> Self {
        unsafe {
            ffi::evm_config_set_block_gas_limit(self.handle, limit);
        }
        self
    }

    /// Set memory initial capacity (default: 4096)
    pub fn memory_initial_capacity(self, capacity: usize) -> Self {
        unsafe {
            ffi::evm_config_set_memory_initial_capacity(self.handle, capacity);
        }
        self
    }

    /// Set memory limit (default: 0xFFFFFF)
    pub fn memory_limit(self, limit: u64) -> Self {
        unsafe {
            ffi::evm_config_set_memory_limit(self.handle, limit);
        }
        self
    }

    /// Set maximum call depth (default: 1024)
    pub fn max_call_depth(self, depth: u16) -> Self {
        unsafe {
            ffi::evm_config_set_max_call_depth(self.handle, depth);
        }
        self
    }

    /// Set loop quota for safety counters
    /// None = disabled, Some(n) = max iterations before panic
    pub fn loop_quota(self, quota: Option<u32>) -> Self {
        unsafe {
            ffi::evm_config_set_loop_quota(self.handle, quota.unwrap_or(0));
        }
        self
    }

    /// Enable or disable system contract features
    pub fn system_contracts(
        self,
        beacon_roots: bool,
        block_hashes: bool,
        deposits: bool,
        withdrawals: bool,
    ) -> Self {
        unsafe {
            ffi::evm_config_enable_system_contracts(
                self.handle,
                beacon_roots,
                block_hashes,
                deposits,
                withdrawals,
            );
        }
        self
    }

    /// Override a specific opcode with a custom handler
    ///
    /// # Arguments
    /// * `opcode` - The opcode byte to override (e.g., 0x01 for ADD)
    /// * `handler` - Closure that receives (frame_ptr, opcode) and returns true if handled
    ///
    /// # Example
    /// ```ignore
    /// let config = EvmConfigBuilder::new()
    ///     .override_opcode(0x01, |_frame_ptr, _opcode| {
    ///         println!("Custom ADD handler");
    ///         true // Handled
    ///     })
    ///     .build();
    /// ```
    pub fn override_opcode<F>(mut self, opcode: u8, handler: F) -> Self
    where
        F: Fn(usize, u8) -> bool + Send + Sync + 'static,
    {
        // Box the closure twice - once for the trait object, once for stable pointer
        let boxed: Box<Box<OpcodeHandlerFn>> = Box::new(Box::new(handler));
        let ctx_ptr = Box::into_raw(boxed) as *mut c_void;

        let success = unsafe {
            ffi::evm_config_add_opcode_override(self.handle, opcode, opcode_trampoline, ctx_ptr)
        };

        if success {
            // Keep the box alive by storing it
            let boxed = unsafe { Box::from_raw(ctx_ptr as *mut Box<OpcodeHandlerFn>) };
            self._opcode_handlers.push(*boxed);
        } else {
            // Clean up on failure
            unsafe {
                let _boxed = Box::from_raw(ctx_ptr as *mut Box<OpcodeHandlerFn>);
            }
            panic!("Failed to add opcode override");
        }

        self
    }

    /// Override or add a custom precompile at a specific address
    ///
    /// # Arguments
    /// * `address` - 20-byte Ethereum address
    /// * `handler` - Closure that receives (address, input, gas_limit) and returns Result
    ///
    /// # Example
    /// ```ignore
    /// use revm::primitives::Address;
    ///
    /// let config = EvmConfigBuilder::new()
    ///     .override_precompile(
    ///         Address::ZERO,
    ///         |addr, input, gas| {
    ///             Ok(PrecompileResult {
    ///                 output: input.to_vec(), // Echo precompile
    ///                 gas_used: 100,
    ///             })
    ///         }
    ///     )
    ///     .build();
    /// ```
    pub fn override_precompile<F>(mut self, address: [u8; 20], handler: F) -> Self
    where
        F: Fn(&[u8], &[u8], u64) -> Result<PrecompileResult, PrecompileError>
            + Send
            + Sync
            + 'static,
    {
        // Box the closure twice - once for the trait object, once for stable pointer
        let boxed: Box<Box<PrecompileHandlerFn>> = Box::new(Box::new(handler));
        let ctx_ptr = Box::into_raw(boxed) as *mut c_void;

        let success = unsafe {
            ffi::evm_config_add_precompile_override(
                self.handle,
                address.as_ptr(),
                precompile_trampoline,
                ctx_ptr,
            )
        };

        if success {
            // Keep the box alive
            let boxed = unsafe { Box::from_raw(ctx_ptr as *mut Box<PrecompileHandlerFn>) };
            self._precompile_handlers.push(*boxed);
        } else {
            // Clean up on failure
            unsafe {
                let _boxed = Box::from_raw(ctx_ptr as *mut Box<PrecompileHandlerFn>);
            }
            panic!("Failed to add precompile override");
        }

        self
    }

    /// Build the final configuration and consume the builder
    /// Returns an EvmConfig that owns the handle
    pub fn build(mut self) -> EvmConfig {
        let handle = self.handle;
        self.handle = std::ptr::null_mut(); // Prevent drop from freeing

        EvmConfig {
            handle,
            _opcode_handlers: std::mem::take(&mut self._opcode_handlers),
            _precompile_handlers: std::mem::take(&mut self._precompile_handlers),
        }
    }
}

impl Drop for EvmConfigBuilder {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::evm_config_destroy(self.handle);
            }
        }
    }
}

impl Default for EvmConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Built EVM configuration (consumed by EVM creation)
pub struct EvmConfig {
    pub(crate) handle: *mut ffi::EvmConfigHandle,
    // Keep handlers alive
    _opcode_handlers: Vec<Box<OpcodeHandlerFn>>,
    _precompile_handlers: Vec<Box<PrecompileHandlerFn>>,
}

impl EvmConfig {
    /// Consume the config and return the raw handle (ownership transferred)
    pub(crate) fn into_raw(mut self) -> *mut ffi::EvmConfigHandle {
        let handle = self.handle;
        self.handle = std::ptr::null_mut(); // Prevent drop
        std::mem::forget(self); // Prevent handler drop
        handle
    }
}

impl Drop for EvmConfig {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::evm_config_destroy(self.handle);
            }
        }
    }
}

// Safety: Config is thread-safe after construction
unsafe impl Send for EvmConfig {}
unsafe impl Sync for EvmConfig {}

// ===== FFI Trampolines =====

/// Trampoline function for opcode handlers
extern "C" fn opcode_trampoline(ctx: *mut c_void, frame_ptr: usize, opcode: u8) -> bool {
    if ctx.is_null() {
        return false;
    }

    // SAFETY: ctx was created by Box::into_raw in override_opcode and points to a valid Box<OpcodeHandlerFn>
    let handler = unsafe { &*(ctx as *const Box<OpcodeHandlerFn>) };
    handler(frame_ptr, opcode)
}

/// Trampoline function for precompile handlers
extern "C" fn precompile_trampoline(
    ctx: *mut c_void,
    address: *const u8,
    input: *const u8,
    input_len: usize,
    gas_limit: u64,
    output_ptr: *mut *mut u8,
    output_len: *mut usize,
    gas_used: *mut u64,
) -> bool {
    if ctx.is_null() {
        return false;
    }

    // SAFETY: ctx was created by Box::into_raw in override_precompile and points to a valid Box<PrecompileHandlerFn>
    let handler = unsafe { &*(ctx as *const Box<PrecompileHandlerFn>) };

    let addr_slice = unsafe { std::slice::from_raw_parts(address, 20) };
    let input_slice = unsafe { std::slice::from_raw_parts(input, input_len) };

    match handler(addr_slice, input_slice, gas_limit) {
        Ok(result) => {
            // Allocate output on heap and transfer ownership to C
            let mut output_vec = result.output;
            output_vec.shrink_to_fit();

            unsafe {
                *output_ptr = output_vec.as_mut_ptr();
                *output_len = output_vec.len();
                *gas_used = result.gas_used;
            }

            // Leak the vec so C side can use it
            std::mem::forget(output_vec);

            true
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder_creation() {
        let config = EvmConfigBuilder::new().build();
        assert!(!config.handle.is_null());
    }

    #[test]
    fn test_config_builder_hardfork() {
        let config = EvmConfigBuilder::new().hardfork("Cancun").build();
        assert!(!config.handle.is_null());
    }

    #[test]
    fn test_config_builder_stack_size() {
        let config = EvmConfigBuilder::new().stack_size(512).build();
        assert!(!config.handle.is_null());
    }

    #[test]
    fn test_config_builder_loop_quota() {
        let config = EvmConfigBuilder::new()
            .loop_quota(Some(1_000_000))
            .build();
        assert!(!config.handle.is_null());
    }
}
