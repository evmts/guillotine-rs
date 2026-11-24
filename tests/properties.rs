use guillotine_rs::guillotine_mini::types::{
    address_from_bytes, address_to_bytes, u256_from_be_bytes, u256_to_be_bytes,
};
use guillotine_rs::GuillotineMiniEvm;
use proptest::prelude::*;
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{address, hardfork::SpecId, Bytes, TxKind, U256},
    state::{AccountInfo, Bytecode},
    Context, MainContext,
};

proptest! {
    #[test]
    fn address_round_trip(bytes in any::<[u8; 20]>()) {
        prop_assert_eq!(address_to_bytes(&address_from_bytes(&bytes)), bytes);
    }

    #[test]
    fn word_round_trip(bytes in any::<[u8; 32]>()) {
        prop_assert_eq!(u256_to_be_bytes(&u256_from_be_bytes(&bytes)), bytes);
    }

    #[test]
    fn evm_add_wraps_modulo_2_to_256(a in any::<[u8; 32]>(), b in any::<[u8; 32]>()) {
        let address = address!("1000000000000000000000000000000000000000");
        let mut code = vec![0x7f]; // PUSH32 a
        code.extend_from_slice(&a);
        code.push(0x7f); // PUSH32 b
        code.extend_from_slice(&b);
        code.extend_from_slice(&[0x01, 0x5f, 0x52, 0x60, 0x20, 0x5f, 0xf3]);

        let mut db = InMemoryDB::default();
        db.insert_account_info(address, AccountInfo::default().with_code(Bytecode::new_raw(Bytes::from(code))));
        let ctx = Context::mainnet().modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN).with_db(db);
        let tx = TxEnv::builder().kind(TxKind::Call(address)).gas_limit(100_000).build().unwrap();
        let result = GuillotineMiniEvm::try_new(ctx).unwrap().transact(tx).unwrap().result;
        prop_assert!(result.is_success());

        let expected = U256::from_be_bytes(a).wrapping_add(U256::from_be_bytes(b)).to_be_bytes::<32>();
        prop_assert_eq!(result.output().unwrap().as_ref(), expected.as_slice());
    }
}
