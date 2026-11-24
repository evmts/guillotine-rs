use keccak_asm::Keccak256;
use std::{ffi::c_uint, slice};

const HASH_LEN: usize = 32;

// These return codes and parameter widths match keccak_wrapper.h on supported targets.
const SUCCESS: c_uint = 0;
const INVALID_INPUT: c_uint = 1;
const INVALID_OUTPUT_SIZE: c_uint = 2;

#[no_mangle]
unsafe extern "C" fn keccak256(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
) -> c_uint {
    if output.is_null() || (input.is_null() && input_len != 0) {
        return INVALID_INPUT;
    }
    if output_len < HASH_LEN {
        return INVALID_OUTPUT_SIZE;
    }
    // The C caller provides readable input and writable output of the given lengths.
    let input = if input_len == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(input, input_len) }
    };
    let hash = Keccak256::digest(input);
    unsafe { output.copy_from_nonoverlapping(hash.as_ptr(), HASH_LEN) };
    SUCCESS
}

#[no_mangle]
unsafe extern "C" fn keccak256_batch(
    inputs: *const *const u8,
    input_lens: *const usize,
    outputs: *const *mut u8,
    count: usize,
) -> c_uint {
    if count == 0 {
        return SUCCESS;
    }
    if inputs.is_null() || input_lens.is_null() || outputs.is_null() {
        return INVALID_INPUT;
    }
    // Each array has count elements and each output holds one 32-byte hash.
    for index in 0..count {
        let result = unsafe {
            keccak256(
                *inputs.add(index),
                *input_lens.add(index),
                *outputs.add(index),
                HASH_LEN,
            )
        };
        if result != SUCCESS {
            return result;
        }
    }
    SUCCESS
}

#[no_mangle]
extern "C" fn keccak256_output_size() -> usize {
    HASH_LEN
}
