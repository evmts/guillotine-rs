//! Integration tests for EVM configuration API

use guillotine_rs::guillotine_mini::{EvmConfigBuilder, GuillotineMiniEvm, PrecompileResult, PrecompileError};
use revm::{
    context::{Context, TxEnv},
    primitives::{Address, Bytes, TxKind, U256},
    MainContext,
};

#[test]
fn test_config_basic_creation() {
    let _config = EvmConfigBuilder::new().build();
    // Config created successfully
}

#[test]
fn test_config_with_hardfork() {
    let _config = EvmConfigBuilder::new().hardfork("Cancun").build();
    // Config created successfully
}

#[test]
fn test_config_with_stack_size() {
    let _config = EvmConfigBuilder::new()
        .stack_size(512)
        .build();
    // Config created successfully
}

#[test]
fn test_config_with_max_call_depth() {
    let _config = EvmConfigBuilder::new()
        .max_call_depth(512)
        .build();
    // Config created successfully
}

#[test]
fn test_config_with_memory_limits() {
    let _config = EvmConfigBuilder::new()
        .memory_initial_capacity(8192)
        .memory_limit(0x1000000)
        .build();
    // Config created successfully
}

#[test]
fn test_config_with_loop_quota() {
    let _config = EvmConfigBuilder::new()
        .loop_quota(Some(1_000_000))
        .build();
    // Config created successfully
}

#[test]
fn test_config_with_system_contracts_disabled() {
    let _config = EvmConfigBuilder::new()
        .system_contracts(false, false, false, false)
        .build();
    // Config created successfully
}

#[test]
fn test_config_chained_builder() {
    let _config = EvmConfigBuilder::new()
        .hardfork("Cancun")
        .stack_size(2048)
        .max_call_depth(2048)
        .memory_limit(0x2000000)
        .loop_quota(Some(5_000_000))
        .system_contracts(true, true, false, false)
        .build();
    // Config created successfully
}

#[test]
fn test_evm_creation_with_config() {
    let ctx = Context::mainnet();
    let config = EvmConfigBuilder::new()
        .hardfork("Cancun")
        .build();

    let result = GuillotineMiniEvm::with_config(ctx, config);
    assert!(result.is_ok());
}

#[test]
fn test_config_with_custom_opcode() {
    let _config = EvmConfigBuilder::new()
        .hardfork("Cancun")
        .override_opcode(0xFF, |_frame_ptr, _opcode| {
            // This won't actually be called in this test, but validates compilation
            true
        })
        .build();
    // Config created successfully with custom opcode
}

#[test]
fn test_config_with_custom_precompile() {
    let _config = EvmConfigBuilder::new()
        .hardfork("Cancun")
        .override_precompile([0u8; 20], |_addr, input, _gas| {
            // Echo precompile: returns input as output
            Ok(PrecompileResult {
                output: input.to_vec(),
                gas_used: 100,
            })
        })
        .build();
    // Config created successfully with custom precompile
}

#[test]
fn test_multiple_opcode_overrides() {
    let _config = EvmConfigBuilder::new()
        .override_opcode(0x01, |_, _| true)
        .override_opcode(0x02, |_, _| true)
        .override_opcode(0x03, |_, _| true)
        .build();
    // Config created successfully with multiple opcode overrides
}

#[test]
fn test_multiple_precompile_overrides() {
    let _config = EvmConfigBuilder::new()
        .override_precompile([1u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], |_, input, _| {
            Ok(PrecompileResult {
                output: input.to_vec(),
                gas_used: 100,
            })
        })
        .override_precompile([2u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], |_, _input, _| {
            Ok(PrecompileResult {
                output: vec![],
                gas_used: 50,
            })
        })
        .build();
    // Config created successfully with multiple precompile overrides
}

#[test]
fn test_config_default_trait() {
    let _config = EvmConfigBuilder::default().build();
    // Config created successfully using default trait
}
