//! Type conversions between REVM and Guillotine-mini
//!
//! Handles conversion between:
//! - REVM's alloy types (Address, U256, Bytes)
//! - Guillotine-mini's C FFI types (byte arrays)

use revm::primitives::{Address, Bytes, U256};

/// Convert REVM Address to 20-byte array for FFI
#[inline]
pub fn address_to_bytes(addr: &Address) -> [u8; 20] {
    addr.0 .0
}

/// Convert 20-byte array from FFI to REVM Address
#[inline]
pub fn address_from_bytes(bytes: &[u8; 20]) -> Address {
    Address::from(*bytes)
}

/// Convert REVM U256 to 32-byte big-endian array for FFI
#[inline]
pub fn u256_to_be_bytes(value: &U256) -> [u8; 32] {
    value.to_be_bytes()
}

/// Convert 32-byte big-endian array from FFI to REVM U256
#[inline]
pub fn u256_from_be_bytes(bytes: &[u8; 32]) -> U256 {
    U256::from_be_bytes(*bytes)
}

/// Convert REVM Bytes to slice for FFI
#[inline]
pub fn bytes_to_slice(bytes: &Bytes) -> &[u8] {
    bytes.as_ref()
}

/// Convert slice to REVM Bytes
#[inline]
pub fn bytes_from_slice(slice: &[u8]) -> Bytes {
    Bytes::copy_from_slice(slice)
}

/// Convert i64 to u64 (gas conversion)
/// Returns 0 if negative
#[inline]
pub fn i64_to_u64_gas(gas: i64) -> u64 {
    gas.max(0) as u64
}

/// Log entry as exported from guillotine-mini
/// Not currently used in the wrapper, but useful for conversions/tests
/// TODO: Enable once log extraction API is finalized
#[allow(dead_code)]
pub struct EvmLog {
    pub address: Address,
    pub topics: Vec<U256>,
    pub data: Bytes,
}

/// Storage change entry captured from guillotine-mini
/// Represents a single slot write in final storage state
/// TODO: Enable once storage change extraction API is finalized
#[allow(dead_code)]
pub struct StorageChange {
    pub address: Address,
    pub slot: U256,
    pub value: U256,
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::primitives::address;

    #[test]
    fn test_address_conversion() {
        let addr = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
        let bytes = address_to_bytes(&addr);
        assert_eq!(bytes.len(), 20);
        assert_eq!(&bytes[..], addr.as_slice());
    }

    #[test]
    fn test_u256_conversion_roundtrip() {
        let original = U256::from(0x123456789abcdef_u64);
        let bytes = u256_to_be_bytes(&original);
        let converted = u256_from_be_bytes(&bytes);
        assert_eq!(original, converted);
    }

    #[test]
    fn test_u256_big_endian() {
        // Value 0x01 should be [0, 0, ..., 0, 1] in big-endian
        let value = U256::from(1);
        let bytes = u256_to_be_bytes(&value);
        assert_eq!(bytes[31], 1);
        assert_eq!(bytes[0], 0);
    }

    #[test]
    fn test_bytes_conversion() {
        let data = Bytes::from(vec![0x60, 0x01, 0x60, 0x02, 0x01]);
        let slice = bytes_to_slice(&data);
        assert_eq!(slice, &[0x60, 0x01, 0x60, 0x02, 0x01]);
    }
}
