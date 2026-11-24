//! Exercise the public adapter and linked crypto from a downstream Cargo target.

use guillotine_rs::GuillotineMiniEvm;
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{address, hardfork::SpecId, Address, Bytes, TxKind},
    state::{AccountInfo, Bytecode},
    Context, MainContext,
};

const CONTRACT: Address = address!("1000000000000000000000000000000000000000");

#[test]
fn keccak_opcode_hashes_calldata() {
    // Copy calldata to memory, hash it, and return the 32-byte digest.
    let code = Bytes::from_static(&[
        0x36, 0x5f, 0x5f, 0x37, 0x36, 0x5f, 0x20, 0x5f, 0x52, 0x60, 0x20, 0x5f, 0xf3,
    ]);
    let mut db = InMemoryDB::default();
    db.insert_account_info(
        CONTRACT,
        AccountInfo::default().with_code(Bytecode::new_raw(code)),
    );
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);
    let mut evm = GuillotineMiniEvm::try_new(ctx).unwrap();
    let tx = TxEnv::builder()
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(100_000)
        .data(Bytes::from_static(b"abc"))
        .build()
        .unwrap();
    let result = evm.transact(tx).unwrap().result;
    assert!(result.is_success(), "{result:?}");
    let expected =
        revm::primitives::b256!("4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45");
    assert_eq!(result.output().unwrap().as_ref(), expected.as_slice());
}

#[test]
fn prague_bls_g1_adds_two_identity_points() {
    // EIP-2537 encodes the point at infinity as 128 zero bytes.
    let g1_add = address!("000000000000000000000000000000000000000b");
    let ctx = Context::mainnet().modify_cfg_chained(|cfg| cfg.spec = SpecId::PRAGUE);
    let mut evm = GuillotineMiniEvm::try_new(ctx).unwrap();
    let tx = TxEnv::builder()
        .kind(TxKind::Call(g1_add))
        .gas_limit(100_000)
        .data(Bytes::from(vec![0; 256]))
        .build()
        .unwrap();
    let result = evm.transact(tx).unwrap().result;
    assert!(result.is_success(), "{result:?}");
    assert_eq!(result.output().unwrap().as_ref(), &[0; 128]);
}

#[test]
fn cancun_kzg_verifies_a_mainnet_proof() {
    // c-kzg-4844 test vector verify_kzg_proof_case_correct_proof_4_4.
    let data = revm::primitives::hex!(
        "01e798154708fe7789429634053cbf9f99b619f9f084048927333fce637f549b"
        "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000"
        "1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9"
        "8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7"
        "a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c"
    );
    let ctx = Context::mainnet().modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN);
    let tx = TxEnv::builder()
        .kind(TxKind::Call(address!(
            "000000000000000000000000000000000000000a"
        )))
        .data(Bytes::copy_from_slice(&data))
        .gas_limit(100_000)
        .build()
        .unwrap();

    let result = GuillotineMiniEvm::try_new(ctx)
        .unwrap()
        .transact(tx)
        .unwrap()
        .result;
    assert!(result.is_success(), "{result:?}");
    assert_eq!(
        result.output().unwrap().as_ref(),
        &revm::primitives::hex!(
            "0000000000000000000000000000000000000000000000000000000000001000"
            "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
        )
    );
}

#[test]
fn storage_clear_reports_original_value_and_preserves_account_info() {
    use revm::{database_interface::DatabaseCommit, primitives::U256};

    // Clear slot zero and leave slot one untouched.
    let code = Bytecode::new_raw(Bytes::from_static(&[0x5f, 0x5f, 0x55, 0x00]));
    let mut db = InMemoryDB::default();
    let mut info = AccountInfo::default().with_code(code);
    info.balance = U256::from(100);
    db.insert_account_info(CONTRACT, info);
    db.insert_account_storage(CONTRACT, U256::ZERO, U256::from(42))
        .unwrap();
    db.insert_account_storage(CONTRACT, U256::from(1), U256::from(9))
        .unwrap();

    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);
    let mut evm = GuillotineMiniEvm::try_new(ctx).unwrap();
    let tx = TxEnv::builder()
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(100_000)
        .build()
        .unwrap();
    let outcome = evm.transact(tx.clone()).unwrap();
    assert!(outcome.result.is_success());
    let account = &outcome.state[&CONTRACT];
    assert_eq!(account.info.balance, U256::from(100));
    assert_eq!(account.storage.len(), 1);
    let slot = &account.storage[&U256::ZERO];
    assert_eq!(slot.original_value, U256::from(42));
    assert_eq!(slot.present_value, U256::ZERO);

    evm.ctx.journaled_state.database.commit(outcome.state);
    let repeated = evm.transact(tx).unwrap();
    assert!(repeated.result.is_success());
    assert!(repeated.state.is_empty());
}

#[test]
fn log_payload_larger_than_four_kib_is_returned_in_full() {
    // LOG0 over 4097 bytes of newly expanded zero-filled memory.
    let code = Bytecode::new_raw(Bytes::from_static(&[0x61, 0x10, 0x01, 0x5f, 0xa0, 0x00]));
    let mut db = InMemoryDB::default();
    db.insert_account_info(CONTRACT, AccountInfo::default().with_code(code));
    let ctx = Context::mainnet()
        .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
        .with_db(db);
    let tx = TxEnv::builder()
        .kind(TxKind::Call(CONTRACT))
        .gas_limit(100_000)
        .build()
        .unwrap();
    let result = GuillotineMiniEvm::try_new(ctx)
        .unwrap()
        .transact(tx)
        .unwrap()
        .result;
    assert!(result.is_success());
    assert_eq!(result.logs()[0].data.data.as_ref(), &[0; 4097]);
}
