//! Minimal test to isolate segfault

use guillotine_rs::guillotine_mini::ffi;
use guillotine_rs::guillotine_mini::types;
use revm::primitives::{address, Address, U256};

#[test]
fn test_ffi_create_only() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };

    assert!(!handle.is_null(), "EVM handle should not be null");
    eprintln!("TEST: EVM handle created successfully");

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}

#[test]
fn test_ffi_set_bytecode() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };

    assert!(!handle.is_null());
    eprintln!("TEST: EVM handle created");

    // Simple bytecode: PUSH1 1 PUSH1 2 ADD PUSH1 0 MSTORE PUSH1 32 PUSH1 0 RETURN
    let bytecode = hex::decode("600160020160005260206000f3").unwrap();
    eprintln!("TEST: Setting bytecode ({} bytes)...", bytecode.len());

    let success = unsafe {
        ffi::evm_set_bytecode(handle, bytecode.as_ptr(), bytecode.len())
    };

    assert!(success, "set_bytecode should succeed");
    eprintln!("TEST: Bytecode set successfully");

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}

#[test]
fn test_ffi_set_execution_context() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };
    assert!(!handle.is_null());
    eprintln!("TEST: EVM handle created");

    let bytecode = hex::decode("600160020160005260206000f3").unwrap();
    eprintln!("TEST: Setting bytecode...");
    let success = unsafe {
        ffi::evm_set_bytecode(handle, bytecode.as_ptr(), bytecode.len())
    };
    assert!(success);
    eprintln!("TEST: Bytecode set");

    // Set execution context
    let caller = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    let contract = address!("1000000000000000000000000000000000000000");
    let value = U256::ZERO;

    eprintln!("TEST: Converting addresses and values...");
    let caller_bytes: [u8; 20] = caller.into();
    let contract_bytes: [u8; 20] = contract.into();
    let value_bytes = value.to_be_bytes::<32>();
    let calldata: &[u8] = &[];

    eprintln!("TEST: Setting execution context...");
    let ctx_success = unsafe {
        ffi::evm_set_execution_context(
            handle,
            100_000,
            caller_bytes.as_ptr(),
            contract_bytes.as_ptr(),
            value_bytes.as_ptr(),
            calldata.as_ptr(),
            calldata.len(),
        )
    };
    assert!(ctx_success, "set_execution_context should succeed");
    eprintln!("TEST: Execution context set");

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}

#[test]
fn test_ffi_set_blockchain_context() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };
    assert!(!handle.is_null());
    eprintln!("TEST: EVM handle created");

    let bytecode = hex::decode("600160020160005260206000f3").unwrap();
    let success = unsafe {
        ffi::evm_set_bytecode(handle, bytecode.as_ptr(), bytecode.len())
    };
    assert!(success);
    eprintln!("TEST: Bytecode set");

    let caller = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    let contract = address!("1000000000000000000000000000000000000000");
    let value = U256::ZERO;
    let caller_bytes: [u8; 20] = caller.into();
    let contract_bytes: [u8; 20] = contract.into();
    let value_bytes = value.to_be_bytes::<32>();
    let calldata: &[u8] = &[];

    let ctx_success = unsafe {
        ffi::evm_set_execution_context(
            handle,
            100_000,
            caller_bytes.as_ptr(),
            contract_bytes.as_ptr(),
            value_bytes.as_ptr(),
            calldata.as_ptr(),
            calldata.len(),
        )
    };
    assert!(ctx_success);
    eprintln!("TEST: Execution context set");

    // Set blockchain context
    let chain_id = U256::from(1);
    let difficulty = U256::ZERO;
    let prevrandao = [0u8; 32];
    let coinbase = Address::ZERO;
    let base_fee = U256::from(1000000000);
    let blob_base_fee = U256::from(1);

    eprintln!("TEST: Converting blockchain context values...");
    let chain_id_bytes = chain_id.to_be_bytes::<32>();
    let difficulty_bytes = difficulty.to_be_bytes::<32>();
    let coinbase_bytes: [u8; 20] = coinbase.into();
    let base_fee_bytes = base_fee.to_be_bytes::<32>();
    let blob_base_fee_bytes = blob_base_fee.to_be_bytes::<32>();

    eprintln!("TEST: Setting blockchain context...");
    unsafe {
        ffi::evm_set_blockchain_context(
            handle,
            chain_id_bytes.as_ptr(),
            1,      // block number
            1000,   // timestamp
            difficulty_bytes.as_ptr(),
            prevrandao.as_ptr(),
            coinbase_bytes.as_ptr(),
            30_000_000, // gas limit
            base_fee_bytes.as_ptr(),
            blob_base_fee_bytes.as_ptr(),
        );
    }
    eprintln!("TEST: Blockchain context set");

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}

#[test]
fn test_ffi_execute() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };
    assert!(!handle.is_null());
    eprintln!("TEST: EVM handle created");

    let bytecode = hex::decode("600160020160005260206000f3").unwrap();
    eprintln!("TEST: Setting bytecode...");
    let success = unsafe {
        ffi::evm_set_bytecode(handle, bytecode.as_ptr(), bytecode.len())
    };
    assert!(success);

    let caller = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
    let contract = address!("1000000000000000000000000000000000000000");
    let value = U256::ZERO;
    let caller_bytes: [u8; 20] = caller.into();
    let contract_bytes: [u8; 20] = contract.into();
    let value_bytes = value.to_be_bytes::<32>();
    let calldata: &[u8] = &[];

    eprintln!("TEST: Setting execution context...");
    let ctx_success = unsafe {
        ffi::evm_set_execution_context(
            handle,
            100_000,
            caller_bytes.as_ptr(),
            contract_bytes.as_ptr(),
            value_bytes.as_ptr(),
            calldata.as_ptr(),
            calldata.len(),
        )
    };
    assert!(ctx_success);

    let chain_id = U256::from(1);
    let difficulty = U256::ZERO;
    let prevrandao = [0u8; 32];
    let coinbase = Address::ZERO;
    let base_fee = U256::from(1000000000);
    let blob_base_fee = U256::from(1);
    let chain_id_bytes = chain_id.to_be_bytes::<32>();
    let difficulty_bytes = difficulty.to_be_bytes::<32>();
    let coinbase_bytes: [u8; 20] = coinbase.into();
    let base_fee_bytes = base_fee.to_be_bytes::<32>();
    let blob_base_fee_bytes = blob_base_fee.to_be_bytes::<32>();

    eprintln!("TEST: Setting blockchain context...");
    unsafe {
        ffi::evm_set_blockchain_context(
            handle,
            chain_id_bytes.as_ptr(),
            1,
            1000,
            difficulty_bytes.as_ptr(),
            prevrandao.as_ptr(),
            coinbase_bytes.as_ptr(),
            30_000_000,
            base_fee_bytes.as_ptr(),
            blob_base_fee_bytes.as_ptr(),
        );
    }

    eprintln!("TEST: Executing transaction...");
    let exec_success = unsafe { ffi::evm_execute(handle) };
    assert!(exec_success, "Execution should succeed");
    eprintln!("TEST: Execution completed");

    eprintln!("TEST: Getting execution results...");
    let gas_used = unsafe { ffi::evm_get_gas_used(handle) };
    let is_success = unsafe { ffi::evm_is_success(handle) };
    eprintln!("TEST: gas_used={}, is_success={}", gas_used, is_success);

    let output_len = unsafe { ffi::evm_get_output_len(handle) };
    eprintln!("TEST: output_len={}", output_len);

    if output_len > 0 {
        let mut output = vec![0u8; output_len];
        unsafe {
            ffi::evm_get_output(handle, output.as_mut_ptr(), output_len);
        }
        eprintln!("TEST: output={:?}", output);
    }

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}

#[test]
fn test_ffi_set_balance_before_execute() {
    eprintln!("TEST: Creating EVM handle...");
    let hardfork = "Cancun";
    let handle = unsafe {
        ffi::evm_create(
            hardfork.as_ptr(),
            hardfork.len(),
            0,
        )
    };
    assert!(!handle.is_null());
    eprintln!("TEST: EVM handle created");

    // Set balance BEFORE setting up execution context
    let test_addr = address!("1000000000000000000000000000000000000000");
    let test_balance = U256::from(1_000_000_u64);

    eprintln!("TEST: Setting balance before execution setup...");
    let addr_bytes = types::address_to_bytes(&test_addr);
    let balance_bytes = types::u256_to_be_bytes(&test_balance);

    unsafe {
        ffi::evm_set_balance(handle, addr_bytes.as_ptr(), balance_bytes.as_ptr());
    }
    eprintln!("TEST: Balance set successfully (no segfault!)");

    // Also test set_code before execution
    let code = hex::decode("600160020160005260206000f3").unwrap();
    eprintln!("TEST: Setting code before execution setup...");
    unsafe {
        ffi::evm_set_code(
            handle,
            addr_bytes.as_ptr(),
            code.as_ptr(),
            code.len(),
        );
    }
    eprintln!("TEST: Code set successfully (no segfault!)");

    unsafe {
        ffi::evm_destroy(handle);
    }
    eprintln!("TEST: EVM handle destroyed");
}
