//! GuillotineMiniEvm - REVM-compatible EVM wrapper
//!
//! Provides a high-level interface to guillotine-mini that integrates with
//! REVM's Database trait for state management.

use super::{database_bridge, ffi, types};
use revm::{
    context::{Cfg, Context, TxEnv},
    context_interface::result::{ExecutionResult, Output, ResultAndState, SuccessReason},
    database_interface::Database,
    primitives::{hardfork::SpecId, Address, Bytes, TxKind, U256},
    state::{Account, AccountInfo, AccountStatus, EvmState, EvmStorageSlot},
};
use std::collections::HashMap;

/// REVM-compatible EVM using guillotine-mini as the execution engine
pub struct GuillotineMiniEvm<CTX> {
    /// REVM context (contains database, config, transaction)
    pub ctx: CTX,
    /// FFI handle to guillotine-mini EVM instance
    handle: *mut ffi::EvmHandle,
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> GuillotineMiniEvm<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>
where
    BLOCK: revm::context_interface::Block,
    TX: revm::context_interface::Transaction,
    CFG: Cfg<Spec = SpecId>,
    DB: Database,
    JOURNAL: revm::context_interface::JournalTr<Database = DB>,
{
    /// Create new GuillotineMiniEvm from REVM context
    pub fn new(ctx: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>) -> Self {
        // Map REVM SpecId to hardfork name
        let hardfork_name = match ctx.cfg.spec() {
            SpecId::FRONTIER | SpecId::FRONTIER_THAWING => "Frontier",
            SpecId::HOMESTEAD | SpecId::DAO_FORK => "Homestead",
            SpecId::TANGERINE => "Tangerine",
            SpecId::SPURIOUS_DRAGON => "Spurious",
            SpecId::BYZANTIUM => "Byzantium",
            SpecId::CONSTANTINOPLE | SpecId::PETERSBURG => "Constantinople",
            SpecId::ISTANBUL | SpecId::MUIR_GLACIER => "Istanbul",
            SpecId::BERLIN => "Berlin",
            SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => "London",
            SpecId::MERGE => "Merge",
            SpecId::SHANGHAI => "Shanghai",
            SpecId::CANCUN => "Cancun",
            SpecId::PRAGUE => "Prague",
            SpecId::OSAKA => "Osaka",
            _ => "Cancun", // Default to Cancun
        };

        // Create guillotine-mini EVM instance
        let handle = unsafe {
            ffi::evm_create(
                hardfork_name.as_ptr(),
                hardfork_name.len(),
                0, // log_level: 0 = none
            )
        };

        assert!(!handle.is_null(), "Failed to create guillotine-mini EVM");

        Self { ctx, handle }
    }

    /// Execute a transaction using guillotine-mini
    pub fn transact(&mut self, tx: TxEnv) -> Result<ResultAndState, DB::Error> {
        // Extract contract address and bytecode
        let (contract_addr, bytecode) = match tx.kind {
            TxKind::Call(addr) => {
                // Get code from database
                let acc = self.ctx.journaled_state.db_mut().basic(addr)?;
                let code = acc
                    .and_then(|a| a.code)
                    .map(|c| c.bytecode().to_vec())
                    .unwrap_or_default();
                (addr, code)
            }
            TxKind::Create => {
                // For CREATE, use provided data as bytecode
                (Address::ZERO, tx.data.to_vec())
            }
        };

        // Sync account pre-state from REVM database to guillotine-mini
        database_bridge::sync_account_to_ffi(self.handle, self.ctx.journaled_state.db_mut(), tx.caller)?;
        database_bridge::sync_account_to_ffi(self.handle, self.ctx.journaled_state.db_mut(), contract_addr)?;

        // Set bytecode
        let bytecode_set = unsafe {
            ffi::evm_set_bytecode(self.handle, bytecode.as_ptr(), bytecode.len())
        };
        assert!(bytecode_set, "Failed to set bytecode");

        // Convert addresses and values to FFI format
        let caller_bytes = types::address_to_bytes(&tx.caller);
        let address_bytes = types::address_to_bytes(&contract_addr);
        let value_bytes = types::u256_to_be_bytes(&tx.value);
        let calldata = types::bytes_to_slice(&tx.data);

        // Set execution context
        let ctx_set = unsafe {
            ffi::evm_set_execution_context(
                self.handle,
                tx.gas_limit as i64,
                caller_bytes.as_ptr(),
                address_bytes.as_ptr(),
                value_bytes.as_ptr(),
                calldata.as_ptr(),
                calldata.len(),
            )
        };
        assert!(ctx_set, "Failed to set execution context");

        // Set blockchain context
        let block = &self.ctx.block;
        let cfg = &self.ctx.cfg;

        let chain_id_bytes = types::u256_to_be_bytes(&U256::from(cfg.chain_id()));
        let difficulty_bytes = types::u256_to_be_bytes(&block.difficulty());
        let prevrandao = block.prevrandao().unwrap_or_default();
        let prevrandao_bytes: [u8; 32] = prevrandao.into();
        let coinbase_bytes = types::address_to_bytes(&block.beneficiary());
        let base_fee_bytes = types::u256_to_be_bytes(&U256::from(block.basefee()));

        // EIP-4844: blob_base_fee
        let blob_base_fee = U256::from(block.blob_gasprice().unwrap_or_default());
        let blob_base_fee_bytes = types::u256_to_be_bytes(&blob_base_fee);

        unsafe {
            ffi::evm_set_blockchain_context(
                self.handle,
                chain_id_bytes.as_ptr(),
                block.number().to::<u64>(),
                block.timestamp().to::<u64>(),
                difficulty_bytes.as_ptr(),
                prevrandao_bytes.as_ptr(),
                coinbase_bytes.as_ptr(),
                block.gas_limit(),
                base_fee_bytes.as_ptr(),
                blob_base_fee_bytes.as_ptr(),
            );
        }

        // Execute
        let success = unsafe { ffi::evm_execute(self.handle) };
        assert!(success, "Execution failed");

        // Get results
        let gas_used = unsafe { ffi::evm_get_gas_used(self.handle) };
        let is_success = unsafe { ffi::evm_is_success(self.handle) };

        // Get output data
        let output_len = unsafe { ffi::evm_get_output_len(self.handle) };
        let mut output_buf = vec![0u8; output_len];
        if output_len > 0 {
            unsafe {
                ffi::evm_get_output(self.handle, output_buf.as_mut_ptr(), output_len);
            }
        }

        // Build execution result
        let output = if is_success {
            Output::Call(Bytes::from(output_buf))
        } else {
            Output::Call(Bytes::default())
        };

        let result = ExecutionResult::Success {
            reason: SuccessReason::Return,
            gas_used: types::i64_to_u64_gas(gas_used),
            gas_refunded: 0, // Guillotine-mini doesn't expose refunds yet
            logs: vec![],     // TODO: Get logs from guillotine-mini
            output,
        };

        // Collect state changes by reading back from guillotine-mini
        // For now, we'll extract storage changes for the contract address
        let mut state = EvmState::default();

        // Read storage changes back
        // We need to track which slots were accessed - for now, check slot 0 and 1
        // (this is a simplified approach; full implementation would track all accessed slots)
        let mut storage_changes = HashMap::new();

        // Check common slots (0, 1, etc.)
        for slot_num in 0..10 {
            let slot = U256::from(slot_num);
            let value = database_bridge::read_storage_from_ffi(self.handle, contract_addr, slot);
            if !value.is_zero() {
                storage_changes.insert(slot, value);
            }
        }

        // Build account state with storage changes
        if !storage_changes.is_empty() || !is_success {
            let mut account = Account {
                info: AccountInfo::default(),
                storage: HashMap::default(),
                status: AccountStatus::Touched,
                transaction_id: 0,
            };

            for (key, value) in storage_changes {
                account.storage.insert(
                    key,
                    EvmStorageSlot {
                        original_value: U256::ZERO,
                        present_value: value,
                        transaction_id: 0,
                        is_cold: false,
                    },
                );
            }

            state.insert(contract_addr, account);
        }

        Ok(ResultAndState { result, state })
    }
}

impl<CTX> Drop for GuillotineMiniEvm<CTX> {
    fn drop(&mut self) {
        unsafe {
            ffi::evm_destroy(self.handle);
        }
    }
}

// Safety: The handle is only used from the same thread
unsafe impl<CTX> Send for GuillotineMiniEvm<CTX> where CTX: Send {}
unsafe impl<CTX> Sync for GuillotineMiniEvm<CTX> where CTX: Sync {}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::MainContext;

    #[test]
    fn test_evm_creation() {
        let ctx = Context::mainnet().modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN);
        let evm = GuillotineMiniEvm::new(ctx);
        // Should not panic, handle created and will be destroyed
        drop(evm);
    }
}
