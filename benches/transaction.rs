use guillotine_rs::GuillotineMiniEvm;
use revm::{
    context::TxEnv,
    database::InMemoryDB,
    primitives::{address, hardfork::SpecId, Bytes, TxKind},
    state::{AccountInfo, Bytecode},
    Context, MainContext,
};
use std::{hint::black_box, time::Instant};

fn main() {
    let contract = address!("1000000000000000000000000000000000000000");
    let mut db = InMemoryDB::default();
    // PUSH1 1, PUSH1 2, ADD, MSTORE, RETURN.
    let code = Bytes::from_static(&[0x60, 1, 0x60, 2, 0x01, 0x5f, 0x52, 0x60, 0x20, 0x5f, 0xf3]);
    db.insert_account_info(
        contract,
        AccountInfo::default().with_code(Bytecode::new_raw(code)),
    );
    let tx = TxEnv::builder()
        .kind(TxKind::Call(contract))
        .gas_limit(100_000)
        .build()
        .unwrap();
    let mut execute = || {
        let ctx = Context::mainnet()
            .modify_cfg_chained(|cfg| cfg.spec = SpecId::CANCUN)
            .with_db(&mut db);
        GuillotineMiniEvm::try_new(ctx)
            .unwrap()
            .transact(black_box(tx.clone()))
            .unwrap()
            .result
    };

    let result = execute();
    assert!(result.is_success());
    assert_eq!(result.output().unwrap()[31], 3);

    let iterations = 10_000;
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(execute());
    }
    let ns_per_tx = start.elapsed().as_nanos() / iterations;
    println!("create and execute simple ADD call ({iterations} iterations) {ns_per_tx} ns/tx");
}
