//! GuillotineMiniEvm - REVM-compatible EVM wrapper
//!
//! Provides a high-level interface to guillotine-mini that integrates with
//! REVM's Database trait for state management.
//!
//! # Known Limitations
//!
//! ## Storage Pre-State Synchronization
//!
//! Storage pre-state is now automatically synchronized for common storage slots (0-9) before
//! execution. This covers most standard contracts (ERC20, ERC721, simple state machines), but
//! has limitations:
//!
//! - **Simple contracts**: Work correctly (slots 0-9 cover most state variables)
//! - **Complex contracts**: May miss storage values in high-numbered slots or dynamic mappings
//! - **Large storage**: Only syncs slots 0-9, not all non-zero slots
//!
//! The current implementation is a temporary solution. Future improvements include:
//! 1. EIP-2930 access list integration to sync exactly the slots that will be accessed
//! 2. On-demand lazy loading via FFI callbacks (requires Zig changes)
//! 3. Heuristics based on contract bytecode analysis
//!
//! You can manually sync additional storage slots before execution using
//! [`database_bridge::sync_storage_to_ffi`](../database_bridge/fn.sync_storage_to_ffi.html) or
//! [`database_bridge::sync_storage_slots_to_ffi`](../database_bridge/fn.sync_storage_slots_to_ffi.html).
//!
//! ## EIP-2930 Access Lists
//!
//! Access list support (EIP-2930) is partially implemented in the FFI layer but not yet integrated
//! into the high-level `transact` method. FFI functions exist (`evm_add_access_list_address`,
//! `evm_add_access_list_storage`) but are not called during transaction execution.
//!
//! **Status**: Planned for future release
//!
//! ## EIP-4844 Blob Transactions
//!
//! Blob transaction support (EIP-4844) is partially implemented:
//!
//! - Blob base fee is set in blockchain context
//! - FFI functions exist for blob hash management
//! - Not yet fully integrated into transaction processing
//!
//! **Status**: Under development
//!
//! ## CREATE2 Nonce Handling
//!
//! The CREATE2 opcode implementation follows REVM's behavior for nonce handling. There may be
//! edge cases where nonce management differs from other EVM implementations. This is inherited
//! behavior from the underlying guillotine-mini engine.
//!
//! ## Error Recovery
//!
//! Catastrophic errors in the Zig layer (panic/unreachable) cause immediate process termination.
//! These cannot be recovered in Rust. Normal execution errors (reverts, out of gas) are properly
//! handled and returned as `ExecutionResult::Revert`.
//!
//! # Examples
//!
//! ## Basic Transaction Execution
//!
//! ```rust,no_run
//! use guillotine_rs::guillotine_mini::GuillotineMiniEvm;
//! use revm::{Context, primitives::{address, TxEnv, TxKind}};
//!
//! let ctx = Context::mainnet();
//! let mut evm = GuillotineMiniEvm::new(ctx);
//!
//! let tx = TxEnv::builder()
//!     .caller(address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"))
//!     .kind(TxKind::Call(address!("0000000000000000000000000000000000000001")))
//!     .gas_limit(100_000)
//!     .build()
//!     .unwrap();
//!
//! let result = evm.transact(tx).unwrap();
//! ```
//!
//! ## Error Handling with try_new
//!
//! ```rust,no_run
//! use guillotine_rs::guillotine_mini::{GuillotineMiniEvm, EvmAdapterError};
//! use revm::Context;
//!
//! let ctx = Context::mainnet();
//! let evm = match GuillotineMiniEvm::try_new(ctx) {
//!     Ok(evm) => evm,
//!     Err(EvmAdapterError::Ffi(name)) => {
//!         eprintln!("FFI call failed: {}", name);
//!         return;
//!     }
//!     Err(EvmAdapterError::Db(e)) => {
//!         eprintln!("Database error: {:?}", e);
//!         return;
//!     }
//! };
//! ```

use super::{database_bridge, error::EvmAdapterError, ffi, types};
use revm::{
    context::{Cfg, Context, TxEnv},
    context_interface::result::{ExecutionResult, Output, ResultAndState, SuccessReason},
    database_interface::Database,
    primitives::{hardfork::SpecId, Address, Bytes, TxKind, U256, B256, Log as RevmLog, LogData},
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

    /// Fallible constructor that returns a proper error instead of panicking
    pub fn try_new(
        ctx: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
    ) -> Result<Self, EvmAdapterError<DB::Error>> {
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
            _ => "Cancun",
        };

        let handle = unsafe { ffi::evm_create(hardfork_name.as_ptr(), hardfork_name.len(), 0) };
        if handle.is_null() {
            return Err(EvmAdapterError::Ffi("evm_create"));
        }
        Ok(Self { ctx, handle })
    }

    // TODO: Re-enable once guillotine-mini upstream adds config FFI functions
    // /// Create new GuillotineMiniEvm with custom configuration
    // ///
    // /// # Arguments
    // /// * `ctx` - REVM context
    // /// * `config` - Custom EVM configuration (consumed)
    // ///
    // /// # Example
    // /// ```ignore
    // /// use guillotine_rs::guillotine_mini::{GuillotineMiniEvm, EvmConfigBuilder};
    // /// use revm::Context;
    // ///
    // /// let config = EvmConfigBuilder::new()
    // ///     .hardfork("Cancun")
    // ///     .stack_size(512)
    // ///     .build();
    // ///
    // /// let evm = GuillotineMiniEvm::with_config(ctx, config).unwrap();
    // /// ```
    // pub fn with_config(
    //     ctx: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
    //     config: EvmConfig,
    // ) -> Result<Self, EvmAdapterError<DB::Error>> {
    //     let config_handle = config.into_raw();
    //
    //     let handle = unsafe { ffi::evm_create_with_config(config_handle, 0) };
    //     if handle.is_null() {
    //         return Err(EvmAdapterError::Ffi("evm_create_with_config"));
    //     }
    //     Ok(Self { ctx, handle })
    // }

    /// Execute a transaction using guillotine-mini
    pub fn transact(&mut self, tx: TxEnv) -> Result<ResultAndState, EvmAdapterError<DB::Error>> {
        // Extract contract address and bytecode
        let (contract_addr, bytecode) = match tx.kind {
            TxKind::Call(addr) => {
                // Get code from database
                let acc = self
                    .ctx
                    .journaled_state
                    .db_mut()
                    .basic(addr)
                    .map_err(EvmAdapterError::Db)?;
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

        // Sync storage pre-state for the contract
        // TODO: Improve storage sync strategy using one of these approaches:
        //   1. EIP-2930 access lists to know exactly which slots to sync
        //   2. On-demand loading via FFI callback mechanism (requires Zig changes)
        //   3. Sync all non-zero slots (expensive for large contracts)
        //   4. Use heuristics based on contract patterns
        //
        // For now, we pre-sync common storage slots (0-9) that are frequently used by:
        //   - Slot 0: Often used for contract state flags or counters
        //   - Slot 1-9: Common for mappings, arrays, and state variables
        //
        // This covers most simple contracts (ERC20, ERC721, etc.) but may miss
        // complex contracts with dynamic storage layouts or high-slot mappings.
        let common_slots: [U256; 10] = [
            U256::from(0),
            U256::from(1),
            U256::from(2),
            U256::from(3),
            U256::from(4),
            U256::from(5),
            U256::from(6),
            U256::from(7),
            U256::from(8),
            U256::from(9),
        ];
        database_bridge::sync_storage_slots_to_ffi(
            self.handle,
            self.ctx.journaled_state.db_mut(),
            contract_addr,
            &common_slots,
        )?;

        // Set bytecode
        let bytecode_set = unsafe { ffi::evm_set_bytecode(self.handle, bytecode.as_ptr(), bytecode.len()) };
        if !bytecode_set {
            return Err(EvmAdapterError::Ffi("evm_set_bytecode"));
        }

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
        if !ctx_set {
            return Err(EvmAdapterError::Ffi("evm_set_execution_context"));
        }

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

        // Execute transaction
        let execute_success = unsafe { ffi::evm_execute(self.handle) };
        if !execute_success {
            return Err(EvmAdapterError::Ffi("evm_execute failed - execution did not complete"));
        }

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

        // Extract gas refund from guillotine-mini
        let gas_refund = unsafe { ffi::evm_get_gas_refund(self.handle) };

        // Extract logs from guillotine-mini
        let log_count = unsafe { ffi::evm_get_log_count(self.handle) };
        let mut logs: Vec<RevmLog> = Vec::with_capacity(log_count);
        for i in 0..log_count {
            let mut log_address = [0u8; 20];
            let mut topics_count: usize = 0;
            let mut topics_buf = [0u8; 128]; // 4 topics * 32 bytes
            let mut data_len: usize = 0;
            let mut data_buf = vec![0u8; 4096];

            let ok = unsafe {
                ffi::evm_get_log(
                    self.handle,
                    i,
                    log_address.as_mut_ptr(),
                    &mut topics_count,
                    topics_buf.as_mut_ptr(),
                    &mut data_len,
                    data_buf.as_mut_ptr(),
                    data_buf.len(),
                )
            };

            if ok {
                let address = types::address_from_bytes(&log_address);
                let mut topics = Vec::with_capacity(topics_count);
                for t in 0..topics_count {
                    let start = t * 32;
                    let end = start + 32;
                    let mut topic_bytes = [0u8; 32];
                    topic_bytes.copy_from_slice(&topics_buf[start..end]);
                    topics.push(B256::from(topic_bytes));
                }
                data_buf.truncate(data_len);
                let log_data = LogData::new(topics, Bytes::from(data_buf)).expect("valid log data");
                logs.push(RevmLog { address, data: log_data });
            }
        }

        let gas_used_u = types::i64_to_u64_gas(gas_used);
        let result = if is_success {
            let output = Output::Call(Bytes::from(output_buf));
            ExecutionResult::Success {
                reason: SuccessReason::Return,
                gas_used: gas_used_u,
                gas_refunded: gas_refund,
                logs,
                output,
            }
        } else {
            ExecutionResult::Revert {
                gas_used: gas_used_u,
                output: Bytes::from(output_buf),
            }
        };

        // Collect state changes by reading back from guillotine-mini
        // For now, we'll extract storage changes for the contract address
        let mut state = EvmState::default();

        // Extract all storage changes from guillotine-mini
        let change_count = unsafe { ffi::evm_get_storage_change_count(self.handle) };
        let mut changes_by_address: HashMap<Address, HashMap<U256, U256>> = HashMap::new();

        for i in 0..change_count {
            let mut addr_bytes = [0u8; 20];
            let mut slot_bytes = [0u8; 32];
            let mut value_bytes = [0u8; 32];
            let ok = unsafe {
                ffi::evm_get_storage_change(
                    self.handle,
                    i,
                    addr_bytes.as_mut_ptr(),
                    slot_bytes.as_mut_ptr(),
                    value_bytes.as_mut_ptr(),
                )
            };
            if ok {
                let addr = types::address_from_bytes(&addr_bytes);
                let slot = types::u256_from_be_bytes(&slot_bytes);
                let value = types::u256_from_be_bytes(&value_bytes);
                changes_by_address
                    .entry(addr)
                    .or_insert_with(HashMap::new)
                    .insert(slot, value);
            }
        }

        // Build account states with actual storage changes
        for (addr, slots) in changes_by_address {
            let mut account = Account {
                info: AccountInfo::default(),
                storage: HashMap::default(),
                status: AccountStatus::Touched,
                transaction_id: 0,
            };

            for (slot, value) in slots {
                account.storage.insert(
                    slot,
                    EvmStorageSlot {
                        original_value: U256::ZERO,
                        present_value: value,
                        transaction_id: 0,
                        is_cold: false,
                    },
                );
            }

            state.insert(addr, account);
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
