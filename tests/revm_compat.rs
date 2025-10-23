//! REVM compatibility tests for guillotine-mini adapter
//! Uses ethereum execution-specs fixtures to verify correctness

use guillotine_rs::GuillotineMiniEvm;
use revm::{
    context::{Context, TxEnv},
    database::{CacheDB, EmptyDB},
    primitives::{address, hardfork::SpecId, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    MainContext,
};

#[test]
fn test_simple_chainid_cancun() {
    // Test from: execution-specs chainid_cancun_state_test_tx_type_0.json
    // Contract code: 0x4660015500
    // Breakdown: CHAINID PUSH1 0x01 SSTORE STOP
    // Should store CHAINID value (0x01) at storage slot 0x01

    let mut db = CacheDB::new(EmptyDB::default());

    // Setup: Contract at 0x1000...
    let contract_addr = address!("1000000000000000000000000000000000000000");
    let code = Bytes::from(hex::decode("4660015500").unwrap());

    // Pre-state: Contract with code, zero balance
    db.insert_account_info(
        contract_addr,
        AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash: revm::primitives::keccak256(&code),
            code: Some(Bytecode::new_raw(code)),
        },
    );

    // Sender account with balance
    let sender = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    db.insert_account_info(
        sender,
        AccountInfo {
            balance: U256::from(0x3635c9adc5dea00000_u128),
            nonce: 0,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
        },
    );

    // Build EVM with guillotine-mini backend
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);
    let mut evm = GuillotineMiniEvm::new(ctx);

    // Execute transaction
    let tx = TxEnv::builder()
        .caller(sender)
        .kind(TxKind::Call(contract_addr))
        .data(Bytes::default())
        .value(U256::ZERO)
        .gas_limit(100_000_000)
        .gas_price(10)
        .build()
        .unwrap();

    let result = evm.transact(tx).unwrap();

    // Verify execution succeeded
    assert!(result.result.is_success(), "Transaction should succeed");

    // Verify state changes
    let state = result.state;

    // Check storage: slot 0x01 should contain 0x01 (CHAINID value)
    let contract_account = state.get(&contract_addr).expect("Contract should exist in state");
    let storage_slot_1 = contract_account.storage.get(&U256::from(1)).expect("Storage slot 0x01 should exist");
    assert_eq!(
        storage_slot_1.present_value,
        U256::from(1),
        "Storage slot 0x01 should contain chain ID (0x01)"
    );

    // Verify gas was consumed
    assert!(result.result.gas_used() > 0, "Should have consumed gas");
}

#[test]
fn test_simple_add() {
    // Simplest possible test: PUSH1 1 PUSH1 2 ADD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
    // Bytecode: 0x600160020160005260206000f3
    // Should return 0x03 (1 + 2)

    let mut db = CacheDB::new(EmptyDB::default());

    let contract_addr = address!("1000000000000000000000000000000000000000");
    let code = Bytes::from(hex::decode("600160020160005260206000f3").unwrap());

    db.insert_account_info(
        contract_addr,
        AccountInfo {
            balance: U256::ZERO,
            nonce: 0,
            code_hash: revm::primitives::keccak256(&code),
            code: Some(Bytecode::new_raw(code)),
        },
    );

    let sender = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    db.insert_account_info(
        sender,
        AccountInfo {
            balance: U256::from(1_000_000_u64),
            nonce: 0,
            code_hash: revm::primitives::KECCAK_EMPTY,
            code: None,
        },
    );

    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);
    let mut evm = GuillotineMiniEvm::new(ctx);

    let tx = TxEnv::builder()
        .caller(sender)
        .kind(TxKind::Call(contract_addr))
        .gas_limit(100_000)
        .build()
        .unwrap();

    let result = evm.transact(tx).unwrap();

    assert!(result.result.is_success());

    // Check return data is 0x03 padded to 32 bytes
    let output = result.result.output().unwrap();
    assert_eq!(output.len(), 32, "Should return 32 bytes");
    assert_eq!(output[31], 3, "Last byte should be 3 (1+2)");
}
