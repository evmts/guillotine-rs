//! FFI bindings to guillotine-mini C API
//!
//! Bindings to lib/guillotine-mini/src/root_c.zig

/// Opaque handle to EVM instance (maps to ExecutionContext in Zig)
#[repr(C)]
pub struct EvmHandle {
    _private: [u8; 0],
}

/// Opaque handle to EVM configuration
#[repr(C)]
pub struct EvmConfigHandle {
    _private: [u8; 0],
}

/// FFI-compatible opcode handler callback
/// Returns true if handled, false to continue with default behavior
pub type FfiOpcodeHandler = extern "C" fn(
    ctx: *mut std::ffi::c_void,
    frame_ptr: usize,
    opcode: u8,
) -> bool;

/// FFI-compatible precompile handler callback
/// Returns true on success, false on failure
pub type FfiPrecompileHandler = extern "C" fn(
    ctx: *mut std::ffi::c_void,
    address: *const u8, // 20 bytes
    input: *const u8,
    input_len: usize,
    gas_limit: u64,
    output_ptr: *mut *mut u8, // Handler sets this to allocated output
    output_len: *mut usize,   // Handler sets output length
    gas_used: *mut u64,       // Handler sets gas consumed
) -> bool;

#[link(name = "guillotine_mini")]
extern "C" {
    // ===== Config Builder API =====

    /// Create a new EVM configuration with default values
    pub fn evm_config_create() -> *mut EvmConfigHandle;

    /// Destroy an EVM configuration
    pub fn evm_config_destroy(handle: *mut EvmConfigHandle);

    /// Set hardfork for the EVM
    pub fn evm_config_set_hardfork(handle: *mut EvmConfigHandle, name: *const u8, len: usize);

    /// Set maximum stack size (default: 1024)
    pub fn evm_config_set_stack_size(handle: *mut EvmConfigHandle, size: u16);

    /// Set maximum bytecode size (default: 24576)
    pub fn evm_config_set_max_bytecode_size(handle: *mut EvmConfigHandle, size: u32);

    /// Set maximum initcode size (default: 49152)
    pub fn evm_config_set_max_initcode_size(handle: *mut EvmConfigHandle, size: u32);

    /// Set block gas limit (default: 30000000)
    pub fn evm_config_set_block_gas_limit(handle: *mut EvmConfigHandle, limit: u64);

    /// Set memory initial capacity (default: 4096)
    pub fn evm_config_set_memory_initial_capacity(handle: *mut EvmConfigHandle, capacity: usize);

    /// Set memory limit (default: 0xFFFFFF)
    pub fn evm_config_set_memory_limit(handle: *mut EvmConfigHandle, limit: u64);

    /// Set maximum call depth (default: 1024)
    pub fn evm_config_set_max_call_depth(handle: *mut EvmConfigHandle, depth: u16);

    /// Set loop quota for safety counters (0 = disabled, >0 = max iterations)
    pub fn evm_config_set_loop_quota(handle: *mut EvmConfigHandle, quota: u32);

    /// Enable or disable system contract features
    pub fn evm_config_enable_system_contracts(
        handle: *mut EvmConfigHandle,
        beacon_roots: bool,
        block_hashes: bool,
        deposits: bool,
        withdrawals: bool,
    );

    /// Add a custom opcode handler override
    /// Returns true on success, false on allocation failure
    pub fn evm_config_add_opcode_override(
        handle: *mut EvmConfigHandle,
        opcode: u8,
        handler: FfiOpcodeHandler,
        ctx: *mut std::ffi::c_void,
    ) -> bool;

    /// Add a custom precompile handler override
    /// Returns true on success, false on allocation failure
    pub fn evm_config_add_precompile_override(
        handle: *mut EvmConfigHandle,
        address_bytes: *const u8, // 20 bytes
        handler: FfiPrecompileHandler,
        ctx: *mut std::ffi::c_void,
    ) -> bool;

    // ===== EVM Creation =====

    /// Create a new EVM instance
    ///
    /// # Parameters
    /// - `hardfork_name`: Hardfork name as C string (e.g., "Cancun")
    /// - `hardfork_len`: Length of hardfork name
    /// - `log_level`: 0=none, 1=err, 2=warn, 3=info, 4=debug
    ///
    /// # Returns
    /// Opaque handle to EVM instance, or null on failure
    pub fn evm_create(
        hardfork_name: *const u8,
        hardfork_len: usize,
        log_level: u8,
    ) -> *mut EvmHandle;

    /// Create a new EVM instance with custom configuration
    /// Config handle is consumed (ownership transferred) and will be freed on evm_destroy
    ///
    /// # Parameters
    /// - `config_handle`: Configuration handle (ownership transferred)
    /// - `log_level`: 0=none, 1=err, 2=warn, 3=info, 4=debug
    ///
    /// # Returns
    /// Opaque handle to EVM instance, or null on failure
    pub fn evm_create_with_config(config_handle: *mut EvmConfigHandle, log_level: u8) -> *mut EvmHandle;

    /// Destroy an EVM instance
    pub fn evm_destroy(handle: *mut EvmHandle);

    /// Set bytecode for execution
    ///
    /// # Returns
    /// true on success, false on allocation failure
    pub fn evm_set_bytecode(
        handle: *mut EvmHandle,
        bytecode: *const u8,
        bytecode_len: usize,
    ) -> bool;

    /// Set execution context (caller, address, value, calldata)
    ///
    /// # Parameters
    /// - `gas`: Gas limit (i64 to allow overflow checking)
    /// - `caller_bytes`: 20-byte caller address
    /// - `address_bytes`: 20-byte contract address
    /// - `value_bytes`: 32-byte value (big-endian u256)
    /// - `calldata`: Input data
    /// - `calldata_len`: Length of input data
    pub fn evm_set_execution_context(
        handle: *mut EvmHandle,
        gas: i64,
        caller_bytes: *const u8,
        address_bytes: *const u8,
        value_bytes: *const u8,
        calldata: *const u8,
        calldata_len: usize,
    ) -> bool;

    /// Set blockchain context (block number, timestamp, coinbase, etc.)
    ///
    /// All u256 parameters are 32-byte big-endian arrays
    pub fn evm_set_blockchain_context(
        handle: *mut EvmHandle,
        chain_id_bytes: *const u8,
        block_number: u64,
        block_timestamp: u64,
        block_difficulty_bytes: *const u8,
        block_prevrandao_bytes: *const u8,
        block_coinbase_bytes: *const u8,
        block_gas_limit: u64,
        block_base_fee_bytes: *const u8,
        blob_base_fee_bytes: *const u8,
    );

    /// Set access list addresses (EIP-2930)
    pub fn evm_set_access_list_addresses(
        handle: *mut EvmHandle,
        addresses: *const u8, // Array of 20-byte addresses
        count: usize,
    ) -> bool;

    /// Set access list storage keys (EIP-2930)
    pub fn evm_set_access_list_storage_keys(
        handle: *mut EvmHandle,
        keys: *const u8, // Array of (address, key) pairs: 20 + 32 = 52 bytes each
        count: usize,
    ) -> bool;

    /// Set blob versioned hashes (EIP-4844)
    pub fn evm_set_blob_hashes(
        handle: *mut EvmHandle,
        hashes: *const u8, // Array of 32-byte hashes
        count: usize,
    ) -> bool;

    /// Execute the transaction
    ///
    /// # Returns
    /// true if execution completed (success or revert), false on error
    pub fn evm_execute(handle: *mut EvmHandle) -> bool;

    /// Get remaining gas after execution
    pub fn evm_get_gas_remaining(handle: *mut EvmHandle) -> i64;

    /// Get gas used during execution
    pub fn evm_get_gas_used(handle: *mut EvmHandle) -> i64;

    /// Check if execution was successful (not reverted)
    pub fn evm_is_success(handle: *mut EvmHandle) -> bool;

    /// Get length of output data
    pub fn evm_get_output_len(handle: *mut EvmHandle) -> usize;

    /// Copy output data to buffer
    ///
    /// # Returns
    /// Number of bytes copied (min of buffer_len and actual output length)
    pub fn evm_get_output(
        handle: *mut EvmHandle,
        buffer: *mut u8,
        buffer_len: usize,
    ) -> usize;

    /// Set storage value (for pre-state setup)
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte contract address
    /// - `key_bytes`: 32-byte storage key (big-endian u256)
    /// - `value_bytes`: 32-byte storage value (big-endian u256)
    pub fn evm_set_storage(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        key_bytes: *const u8,
        value_bytes: *const u8,
    ) -> bool;

    /// Get storage value
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte contract address
    /// - `key_bytes`: 32-byte storage key (big-endian u256)
    /// - `value_bytes`: Output buffer for 32-byte storage value (big-endian u256)
    pub fn evm_get_storage(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        key_bytes: *const u8,
        value_bytes: *mut u8,
    ) -> bool;

    /// Set account balance (for pre-state setup)
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte account address
    /// - `balance_bytes`: 32-byte balance (big-endian u256)
    pub fn evm_set_balance(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        balance_bytes: *const u8,
    ) -> bool;

    /// Set account code (for pre-state setup)
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte account address
    /// - `code`: Bytecode
    /// - `code_len`: Length of bytecode
    pub fn evm_set_code(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        code: *const u8,
        code_len: usize,
    ) -> bool;

    /// Set account nonce (for pre-state setup)
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte account address
    /// - `nonce`: Nonce value
    ///
    /// # Returns
    /// true on success, false on failure
    pub fn evm_set_nonce(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        nonce: u64,
    ) -> bool;

    // ===== Added: Result introspection (logs, refunds, storage changes) =====

    /// Get number of log entries in the last execution
    pub fn evm_get_log_count(handle: *mut EvmHandle) -> usize;

    /// Get a log entry by index. Returns true on success.
    /// - `address_out`: 20-byte buffer
    /// - `topics_count_out`: number of topics returned
    /// - `topics_out`: buffer for up to 4 topics (4 * 32 bytes)
    /// - `data_len_out`: actual data length
    /// - `data_out`: data buffer
    /// - `data_max_len`: capacity of `data_out`
    pub fn evm_get_log(
        handle: *mut EvmHandle,
        index: usize,
        address_out: *mut u8,
        topics_count_out: *mut usize,
        topics_out: *mut u8,
        data_len_out: *mut usize,
        data_out: *mut u8,
        data_max_len: usize,
    ) -> bool;

    /// Get gas refund counter after execution
    pub fn evm_get_gas_refund(handle: *mut EvmHandle) -> u64;

    /// Get number of storage changes (entries present in storage map)
    pub fn evm_get_storage_change_count(handle: *mut EvmHandle) -> usize;

    /// Get storage change by index. Returns true on success.
    /// - `address_out`: 20-byte buffer
    /// - `slot_out`: 32-byte buffer (big-endian u256)
    /// - `value_out`: 32-byte buffer (big-endian u256)
    pub fn evm_get_storage_change(
        handle: *mut EvmHandle,
        index: usize,
        address_out: *mut u8,
        slot_out: *mut u8,
        value_out: *mut u8,
    ) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_create_destroy() {
        unsafe {
            let handle = evm_create(b"Cancun".as_ptr(), 6, 0);
            assert!(!handle.is_null(), "Failed to create EVM handle");
            evm_destroy(handle);
        }
    }
}
