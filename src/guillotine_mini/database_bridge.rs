//! Bridge between REVM's Database trait and guillotine-mini's FFI
//!
//! This module handles synchronizing state between REVM's CacheDB and
//! guillotine-mini's internal storage via FFI calls.

use super::ffi::EvmHandle;
use super::types::{address_to_bytes, u256_to_be_bytes};
use revm::database_interface::Database;
use revm::primitives::{Address, U256};

/// Synchronize account state from REVM Database to guillotine-mini
///
/// Sets up pre-state in guillotine-mini before execution
pub fn sync_account_to_ffi<DB: Database>(
    handle: *mut EvmHandle,
    db: &mut DB,
    address: Address,
) -> Result<(), DB::Error> {
    let addr_bytes = address_to_bytes(&address);

    // Get account info from REVM database
    let acc = db.basic(address)?;

    if let Some(acc_info) = acc {
        // Set balance
        let balance_bytes = u256_to_be_bytes(&acc_info.balance);
        unsafe {
            super::ffi::evm_set_balance(handle, addr_bytes.as_ptr(), balance_bytes.as_ptr());
        }

        // Set nonce
        let nonce_set = unsafe {
            super::ffi::evm_set_nonce(handle, addr_bytes.as_ptr(), acc_info.nonce)
        };
        assert!(nonce_set, "Failed to set nonce for address {:?}", address);

        // Set code if exists
        if let Some(code) = &acc_info.code {
            let code_bytes = code.bytecode();
            unsafe {
                super::ffi::evm_set_code(
                    handle,
                    addr_bytes.as_ptr(),
                    code_bytes.as_ptr(),
                    code_bytes.len(),
                );
            }
        }
    }

    Ok(())
}

/// Synchronize storage slot from REVM Database to guillotine-mini
pub fn sync_storage_to_ffi<DB: Database>(
    handle: *mut EvmHandle,
    db: &mut DB,
    address: Address,
    slot: U256,
) -> Result<(), DB::Error> {
    let addr_bytes = address_to_bytes(&address);
    let key_bytes = u256_to_be_bytes(&slot);

    // Get storage value from REVM database
    let value = db.storage(address, slot)?;
    let value_bytes = u256_to_be_bytes(&value);

    unsafe {
        super::ffi::evm_set_storage(
            handle,
            addr_bytes.as_ptr(),
            key_bytes.as_ptr(),
            value_bytes.as_ptr(),
        );
    }

    Ok(())
}

/// Read storage value back from guillotine-mini FFI
pub fn read_storage_from_ffi(handle: *mut EvmHandle, address: Address, slot: U256) -> U256 {
    let addr_bytes = address_to_bytes(&address);
    let key_bytes = u256_to_be_bytes(&slot);
    let mut value_bytes = [0u8; 32];

    unsafe {
        super::ffi::evm_get_storage(
            handle,
            addr_bytes.as_ptr(),
            key_bytes.as_ptr(),
            value_bytes.as_mut_ptr(),
        );
    }

    super::types::u256_from_be_bytes(&value_bytes)
}
