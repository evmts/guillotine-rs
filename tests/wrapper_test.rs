//! Test GuillotineMiniEvm wrapper in isolation

use guillotine_rs::GuillotineMiniEvm;
use revm::{
    context::Context,
    database::{CacheDB, EmptyDB},
    primitives::{address, hardfork::SpecId, Address, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    MainContext,
};

#[test]
fn test_wrapper_create() {
    eprintln!("WRAPPER TEST: Creating context...");
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN);

    eprintln!("WRAPPER TEST: Creating GuillotineMiniEvm...");
    let evm = GuillotineMiniEvm::new(ctx);

    eprintln!("WRAPPER TEST: GuillotineMiniEvm created successfully");
    drop(evm);
    eprintln!("WRAPPER TEST: GuillotineMiniEvm dropped");
}

#[test]
fn test_wrapper_with_database() {
    eprintln!("WRAPPER TEST: Creating database...");
    let mut db = CacheDB::new(EmptyDB::default());

    let contract_addr = address!("1000000000000000000000000000000000000000");
    let code = Bytes::from(hex::decode("600160020160005260206000f3").unwrap());

    eprintln!("WRAPPER TEST: Inserting contract into database...");
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

    eprintln!("WRAPPER TEST: Creating context with database...");
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);

    eprintln!("WRAPPER TEST: Creating GuillotineMiniEvm...");
    let evm = GuillotineMiniEvm::new(ctx);

    eprintln!("WRAPPER TEST: GuillotineMiniEvm created successfully");
    drop(evm);
    eprintln!("WRAPPER TEST: GuillotineMiniEvm dropped");
}

#[test]
fn test_wrapper_transact_simple() {
    use revm::context::TxEnv;

    eprintln!("WRAPPER TEST: Creating database...");
    let mut db = CacheDB::new(EmptyDB::default());

    let contract_addr = address!("1000000000000000000000000000000000000000");
    let code = Bytes::from(hex::decode("600160020160005260206000f3").unwrap());

    eprintln!("WRAPPER TEST: Inserting contract into database...");
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

    eprintln!("WRAPPER TEST: Creating context...");
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);

    eprintln!("WRAPPER TEST: Creating GuillotineMiniEvm...");
    let mut evm = GuillotineMiniEvm::new(ctx);

    eprintln!("WRAPPER TEST: Building transaction...");
    let tx = TxEnv::builder()
        .caller(sender)
        .kind(TxKind::Call(contract_addr))
        .gas_limit(100_000)
        .build()
        .unwrap();

    eprintln!("WRAPPER TEST: Executing transact()...");
    let result = evm.transact(tx);

    eprintln!("WRAPPER TEST: transact() returned: {:?}", result.is_ok());

    if let Ok(r) = result {
        eprintln!("WRAPPER TEST: Success! gas_used={:?}, is_success={}",
                  r.result.gas_used(), r.result.is_success());
    } else {
        eprintln!("WRAPPER TEST: Error: {:?}", result.err());
    }
}
