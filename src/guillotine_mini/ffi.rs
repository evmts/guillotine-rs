//! FFI bindings to guillotine-mini C API
//!
//! Bindings to lib/guillotine-mini/src/root_c.zig

/// Opaque handle to EVM instance (maps to ExecutionContext in Zig)
#[repr(C)]
pub struct EvmHandle {
    _private: [u8; 0],
}

#[link(name = "guillotine_mini")]
extern "C" {
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
    );

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
    );

    /// Set account balance (for pre-state setup)
    ///
    /// # Parameters
    /// - `address_bytes`: 20-byte account address
    /// - `balance_bytes`: 32-byte balance (big-endian u256)
    pub fn evm_set_balance(
        handle: *mut EvmHandle,
        address_bytes: *const u8,
        balance_bytes: *const u8,
    );

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
    );

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
