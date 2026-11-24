//! GuillotineMiniEvm - REVM-compatible EVM wrapper
//!
//! Provides a high-level interface to guillotine-mini that integrates with
//! REVM's Database trait for state management.
//!
//! The adapter executes calls and returns native execution gas, output, logs,
//! and changed storage slots. It seeds caller and destination account data,
//! including destination storage slots 0 through 9.
//!
//! Complete transaction accounting and dynamic state loading are tracked in
//! [issue #2](https://github.com/evmts/guillotine-rs/issues/2).
//!
//! # Examples
//!
//! ## Basic Transaction Execution
//!
//! ```rust,no_run
//! use guillotine_rs::guillotine_mini::GuillotineMiniEvm;
//! use revm::{context::TxEnv, primitives::{address, TxKind}, Context, MainContext};
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
//! use revm::{Context, MainContext};
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
    context::{Context, TxEnv},
    context_interface::{
        result::{ExecutionResult, Output, ResultAndState, SuccessReason},
        Cfg,
    },
    database_interface::Database,
    primitives::{hardfork::SpecId, Address, Bytes, Log as RevmLog, LogData, TxKind, B256, U256},
    state::{Account, EvmState, EvmStorageSlot},
};

/// REVM-compatible EVM using guillotine-mini as the execution engine
pub struct GuillotineMiniEvm<CTX> {
    /// REVM context (contains database, config, transaction)
    pub ctx: CTX,
    /// FFI handle to guillotine-mini EVM instance
    handle: *mut ffi::EvmHandle,
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
    GuillotineMiniEvm<Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>>
where
    BLOCK: revm::context_interface::Block,
    TX: revm::context_interface::Transaction,
    CFG: Cfg<Spec = SpecId>,
    DB: Database,
    JOURNAL: revm::context_interface::JournalTr<Database = DB>,
{
    /// Create a new adapter using the hardfork selected by the REVM context.
    ///
    /// # Panics
    ///
    /// Panics for an unsupported hardfork or a failed native allocation. Use [Self::try_new]
    /// to handle the error.
    pub fn new(ctx: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>) -> Self {
        Self::try_new(ctx).unwrap_or_else(|_| panic!("Failed to create guillotine-mini EVM"))
    }

    /// Create a new adapter using the hardfork selected by the REVM context.
    ///
    /// # Errors
    ///
    /// Returns an FFI error for an unsupported hardfork or a failed native allocation.
    pub fn try_new(
        ctx: Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>,
    ) -> Result<Self, EvmAdapterError<DB::Error>> {
        // Map REVM SpecId to hardfork name
        let hardfork_name = match ctx.cfg.spec() {
            SpecId::FRONTIER | SpecId::FRONTIER_THAWING => "Frontier",
            SpecId::HOMESTEAD => "Homestead",
            SpecId::DAO_FORK => "DAO",
            SpecId::TANGERINE => "TangerineWhistle",
            SpecId::SPURIOUS_DRAGON => "SpuriousDragon",
            SpecId::BYZANTIUM => "Byzantium",
            SpecId::CONSTANTINOPLE => "Constantinople",
            SpecId::PETERSBURG => "Petersburg",
            SpecId::ISTANBUL => "Istanbul",
            SpecId::MUIR_GLACIER => "MuirGlacier",
            SpecId::BERLIN => "Berlin",
            SpecId::LONDON => "London",
            SpecId::ARROW_GLACIER => "ArrowGlacier",
            SpecId::GRAY_GLACIER => "GrayGlacier",
            SpecId::MERGE => "Merge",
            SpecId::SHANGHAI => "Shanghai",
            SpecId::CANCUN => "Cancun",
            SpecId::PRAGUE => "Prague",
            SpecId::OSAKA => "Osaka",
            _ => return Err(EvmAdapterError::Ffi("unsupported hardfork")),
        };

        let handle = unsafe { ffi::evm_create(hardfork_name.as_ptr(), hardfork_name.len(), 0) };
        if handle.is_null() {
            return Err(EvmAdapterError::Ffi("evm_create"));
        }
        Ok(Self { ctx, handle })
    }

    /// Execute a call using the current database and block context.
    ///
    /// The result includes changed storage slots. The caller controls whether
    /// to commit the changes to its database. A native execution failure with
    /// a result is mapped to a REVM revert.
    ///
    /// # Errors
    ///
    /// Returns a database or FFI error if pre-state, execution, or result
    /// extraction fails.
    pub fn transact(&mut self, tx: TxEnv) -> Result<ResultAndState, EvmAdapterError<DB::Error>> {
        let gas_limit = i64::try_from(tx.gas_limit)
            .map_err(|_| EvmAdapterError::Ffi("gas limit exceeds the native range"))?;

        // SAFETY: this adapter owns the live handle and transact takes an exclusive borrow.
        unsafe { ffi::evm_reset(self.handle) };

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
                let code = match acc {
                    Some(info) => match info.code {
                        Some(code) => Some(code),
                        None if info.code_hash != revm::primitives::KECCAK_EMPTY => Some(
                            self.ctx
                                .journaled_state
                                .db_mut()
                                .code_by_hash(info.code_hash)
                                .map_err(EvmAdapterError::Db)?,
                        ),
                        None => None,
                    },
                    None => None,
                };
                let code = code
                    .map(|code| code.original_byte_slice().to_vec())
                    .unwrap_or_default();
                (addr, code)
            }
            TxKind::Create => {
                // For CREATE, use provided data as bytecode
                (Address::ZERO, tx.data.to_vec())
            }
        };

        // Sync account pre-state from REVM database to guillotine-mini
        // SAFETY: both calls have exclusive access to the adapter's live handle.
        unsafe {
            let db = self.ctx.journaled_state.db_mut();
            database_bridge::sync_account_to_ffi(self.handle, db, tx.caller)?;
            database_bridge::sync_account_to_ffi(self.handle, db, contract_addr)?;
        }

        // Sync storage pre-state for the contract
        // Dynamic state loading is tracked in https://github.com/evmts/guillotine-rs/issues/2.
        let common_slots: [U256; 10] = std::array::from_fn(U256::from);
        // SAFETY: the adapter owns the handle and has exclusive access here.
        unsafe {
            database_bridge::sync_storage_slots_to_ffi(
                self.handle,
                self.ctx.journaled_state.db_mut(),
                contract_addr,
                &common_slots,
            )?;
        }

        // Set bytecode
        let bytecode_set =
            unsafe { ffi::evm_set_bytecode(self.handle, bytecode.as_ptr(), bytecode.len()) };
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
                gas_limit,
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
        if !unsafe { ffi::evm_execute(self.handle) || ffi::evm_has_result(self.handle) } {
            return Err(EvmAdapterError::Ffi(
                "evm_execute failed - execution did not complete",
            ));
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
            let mut data_buf = Vec::new();

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

            if !ok {
                return Err(EvmAdapterError::Ffi("evm_get_log"));
            }
            if data_len > 0 {
                data_buf.resize(data_len, 0);
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
                if !ok {
                    return Err(EvmAdapterError::Ffi("evm_get_log"));
                }
            }
            let address = types::address_from_bytes(&log_address);
            let topics = topics_buf
                .chunks_exact(32)
                .take(topics_count)
                .map(B256::from_slice)
                .collect();
            let data = LogData::new(topics, Bytes::from(data_buf))
                .ok_or(EvmAdapterError::Ffi("invalid topics from evm_get_log"))?;
            logs.push(RevmLog { address, data });
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

        let mut state = EvmState::default();
        if is_success {
            let change_count = unsafe { ffi::evm_get_storage_change_count(self.handle) };
            let db = self.ctx.journaled_state.db_mut();
            for index in 0..change_count {
                let mut addr_bytes = [0; 20];
                let mut slot_bytes = [0; 32];
                let mut value_bytes = [0; 32];
                if !unsafe {
                    ffi::evm_get_storage_change(
                        self.handle,
                        index,
                        addr_bytes.as_mut_ptr(),
                        slot_bytes.as_mut_ptr(),
                        value_bytes.as_mut_ptr(),
                    )
                } {
                    return Err(EvmAdapterError::Ffi("evm_get_storage_change"));
                }
                let addr = types::address_from_bytes(&addr_bytes);
                let slot = types::u256_from_be_bytes(&slot_bytes);
                let value = types::u256_from_be_bytes(&value_bytes);
                let original = db.storage(addr, slot).map_err(EvmAdapterError::Db)?;
                if original == value {
                    continue;
                }
                let account = match state.entry(addr) {
                    std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let info = db
                            .basic(addr)
                            .map_err(EvmAdapterError::Db)?
                            .unwrap_or_default();
                        entry.insert(Account::from(info))
                    }
                };
                account.mark_touch();
                account
                    .storage
                    .insert(slot, EvmStorageSlot::new_changed(original, value, 0));
            }
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
