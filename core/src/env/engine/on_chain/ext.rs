// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! External C API to communicate with substrate contracts runtime module.
//!
//! Refer to substrate SRML contract module for more documentation.

use ink_primitives::Key;
use crate::env::ReturnFlags;

macro_rules! define_error_codes {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident = $discr:literal,
        )*
    ) => {
        /// Either success or any error that can be returned from a runtime API call.
        #[repr(u32)]
        pub enum ReturnCode {
            /// API call successful.
            Success = 0,
            $(
                $( #[$attr] )*
                $name = $discr,
            )*
            /// Returns if an unknown error was received from the host module.
            UnknownError,
        }

        impl From<u32> for ReturnCode {
            /// Returns a new return code from the given raw value if valid.
            ///
            /// Returns `None` if the raw value is not a valid discriminant.
            #[inline]
            fn from(raw: u32) -> Self {
                match raw {
                    0 => Self::Success,
                    $(
                        $discr => Self::$name,
                    )*
                    _ => Self::UnknownError,
                }
            }
        }

        /// Every error that can be returned from a runtime API call.
        #[repr(u32)]
        pub enum Error {
            $(
                $( #[$attr] )*
                $name = $discr,
            )*
            /// Returns if an unknown error was received from the host module.
            UnknownError,
        }

        impl From<ReturnCode> for Result {
            #[inline]
            fn from(return_code: ReturnCode) -> Self {
                match return_code {
                    ReturnCode::Success => Ok(()),
                    $(
                        ReturnCode::$name => Err(Error::$name),
                    )*
                    ReturnCode::UnknownError => Err(Error::UnknownError),
                }
            }
        }
    };
}
define_error_codes! {
    /// The called function trapped and has its state changes reverted.
    /// In this case no output buffer is returned.
    /// Can only be returned from `ext_call` and `ext_instantiate`.
    CalleeTrapped = 1,
    /// The called function ran to completion but decided to revert its state.
    /// An output buffer is returned when one was supplied.
    /// Can only be returned from `ext_call` and `ext_instantiate`.
    CalleeReverted = 2,
    /// The passed key does not exist in storage.
    KeyNotFound = 3,
}

type Result = core::result::Result<(), Error>;

mod sys {
    extern "C" {
        pub fn ext_instantiate(
            init_code_ptr: u32,
            init_code_len: u32,
            gas: u64,
            endowment_ptr: u32,
            endowment_len: u32,
            input_ptr: u32,
            input_len: u32,
            address_ptr: u32,
            address_len_ptr: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> u32;

        pub fn ext_call(
            callee_ptr: u32,
            callee_len: u32,
            gas: u64,
            transferred_value_ptr: u32,
            transferred_value_len: u32,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> u32;

        pub fn ext_transfer(
            account_id_ptr: u32,
            account_id_len: u32,
            transferred_value_ptr: u32,
            transferred_value_len: u32,
        );

        pub fn ext_deposit_event(
            topics_ptr: u32,
            topics_len: u32,
            data_ptr: u32,
            data_len: u32,
        );

        pub fn ext_set_storage(key_ptr: u32, value_ptr: u32, value_len: u32);
        pub fn ext_get_storage(key_ptr: u32, output_ptr: u32, output_len_ptr: u32)
            -> u32;
        pub fn ext_clear_storage(key_ptr: u32);

        pub fn ext_restore_to(
            dest_ptr: u32,
            dest_len: u32,
            code_hash_ptr: u32,
            code_hash_len: u32,
            rent_allowance_ptr: u32,
            rent_allowance_len: u32,
            delta_ptr: u32,
            delta_count: u32,
        );
        pub fn ext_terminate(beneficiary_ptr: u32, beneficiary_len: u32) -> !;

        pub fn ext_call_chain_extension(
            func_id: u32,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> u32;

        pub fn ext_input(buf_ptr: u32, buf_len_ptr: u32);
        pub fn ext_return(flags: u32, data_ptr: u32, data_len: u32) -> !;

        pub fn ext_caller(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_block_number(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_address(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_balance(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_weight_to_fee(gas: u64, output_ptr: u32, output_len_ptr: u32);
        pub fn ext_gas_left(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_value_transferred(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_now(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_rent_allowance(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_minimum_balance(output_ptr: u32, output_len_ptr: u32);
        pub fn ext_tombstone_deposit(output_ptr: u32, output_len_ptr: u32);

        pub fn ext_set_rent_allowance(value_ptr: u32, value_len: u32);

        pub fn ext_random(
            subject_ptr: u32,
            subject_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        );
        pub fn ext_println(str_ptr: u32, str_len: u32);

        pub fn ext_hash_keccak_256(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn ext_hash_blake2_256(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn ext_hash_blake2_128(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn ext_hash_sha2_256(input_ptr: u32, input_len: u32, output_ptr: u32);
    }
}

fn extract_from_slice(output: &mut &mut [u8], new_len: usize) {
    debug_assert!(new_len <= output.len());
    let tmp = core::mem::take(output);
    *output = &mut tmp[..new_len];
}

pub fn instantiate(
    code_hash: &[u8],
    gas_limit: u64,
    endowment: &[u8],
    input: &[u8],
    out_address: &mut &mut [u8],
    out_return_value: &mut &mut [u8],
) -> Result {
    let mut address_len = out_address.len() as u32;
    let mut return_value_len = out_return_value.len() as u32;
    let ret_code = {
        let address_len_ptr: *mut u32 = &mut address_len;
        let return_value_len_ptr: *mut u32 = &mut return_value_len;
        unsafe {
            sys::ext_instantiate(
                code_hash.as_ptr() as u32,
                code_hash.len() as u32,
                gas_limit,
                endowment.as_ptr() as u32,
                endowment.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                out_address.as_ptr() as u32,
                address_len_ptr as u32,
                out_return_value.as_ptr() as u32,
                return_value_len_ptr as u32,
            )
        }
    };
    extract_from_slice(out_address, address_len as usize);
    extract_from_slice(out_return_value, return_value_len as usize);
    ReturnCode::from(ret_code).into()
}

pub fn call(
    callee: &[u8],
    gas_limit: u64,
    value: &[u8],
    input: &[u8],
    output: &mut &mut [u8],
) -> Result {
    let mut output_len = output.len() as u32;
    let ret_code = {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::ext_call(
                callee.as_ptr() as u32,
                callee.len() as u32,
                gas_limit,
                value.as_ptr() as u32,
                value.len() as u32,
                input.as_ptr() as u32,
                input.len() as u32,
                output.as_ptr() as u32,
                output_len_ptr as u32,
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ReturnCode::from(ret_code).into()
}

pub fn transfer(account_id: &[u8], value: &[u8]) {
    unsafe {
        sys::ext_transfer(
            account_id.as_ptr() as u32,
            account_id.len() as u32,
            value.as_ptr() as u32,
            value.len() as u32,
        )
    }
}

pub fn deposit_event(topics: &[u8], data: &[u8]) {
    unsafe {
        sys::ext_deposit_event(
            topics.as_ptr() as u32,
            topics.len() as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    }
}

pub fn set_storage(key: &[u8], encoded_value: &[u8]) {
    unsafe {
        sys::ext_set_storage(
            key.as_ptr() as u32,
            encoded_value.as_ptr() as u32,
            encoded_value.len() as u32,
        )
    }
}

pub fn clear_storage(key: &[u8]) {
    unsafe { sys::ext_clear_storage(key.as_ptr() as u32) }
}

pub fn get_storage(key: &[u8], output: &mut &mut [u8]) -> Result {
    let mut output_len = output.len() as u32;
    let ret_code = {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::ext_get_storage(
                key.as_ptr() as u32,
                output.as_ptr() as u32,
                output_len_ptr as u32,
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ReturnCode::from(ret_code).into()
}

/// Restores a tombstone to the original smart contract.
///
/// # Params
///
/// - `account_id`: Encoded bytes of the `AccountId` of the to-be-restored contract.
/// - `code_hash`: Encoded code hash of the to-be-restored contract.
/// - `rent_allowance`: The encoded rent allowance of the restored contract
///                     upon successful restoration.
/// - `filtered_keys`: Storage keys that will be ignored for the tombstone hash
///                    match calculation that decide whether the original contract
///                    storage and the storage of the restorer contract is equal.
pub fn restore_to(
    account_id: &[u8],
    code_hash: &[u8],
    rent_allowance: &[u8],
    filtered_keys: &[Key],
) {
    unsafe {
        sys::ext_restore_to(
            account_id.as_ptr() as u32,
            account_id.len() as u32,
            code_hash.as_ptr() as u32,
            code_hash.len() as u32,
            rent_allowance.as_ptr() as u32,
            rent_allowance.len() as u32,
            filtered_keys.as_ptr() as u32,
            filtered_keys.len() as u32,
        )
    }
}

pub fn terminate(beneficiary: &[u8]) -> ! {
    unsafe { sys::ext_terminate(beneficiary.as_ptr() as u32, beneficiary.len() as u32) }
}

pub fn call_chain_extension(
    func_id: u32,
    input: &[u8],
    output: &mut &mut [u8],
) -> Result {
    let mut output_len = output.len() as u32;
    let ret_code = {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::ext_call_chain_extension(
                func_id,
                input.as_ptr() as u32,
                input.len() as u32,
                output.as_ptr() as u32,
                output_len_ptr as u32,
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ReturnCode::from(ret_code).into()
}

pub fn input(output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe { sys::ext_input(output.as_ptr() as u32, output_len_ptr as u32) };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
    unsafe {
        sys::ext_return(
            flags.into_u32(),
            return_value.as_ptr() as u32,
            return_value.len() as u32,
        )
    }
}

macro_rules! impl_ext_wrapper_for {
    ( $( ($name:ident => $ext_name:ident), )* ) => {
        $(
            pub fn $name(output: &mut &mut [u8]) {
                let mut output_len = output.len() as u32;
                {
                    let output_len_ptr: *mut u32 = &mut output_len;
                    unsafe {
                        sys::$ext_name(
                            output.as_ptr() as u32,
                            output_len_ptr as u32,
                        )
                    };
                }
                extract_from_slice(output, output_len as usize);
            }
        )*
    }
}
impl_ext_wrapper_for! {
    (caller => ext_caller),
    (block_number => ext_block_number),
    (address => ext_address),
    (balance => ext_balance),
    (gas_left => ext_gas_left),
    (value_transferred => ext_value_transferred),
    (now => ext_now),
    (rent_allowance => ext_rent_allowance),
    (minimum_balance => ext_minimum_balance),
    (tombstone_deposit => ext_tombstone_deposit),
}

pub fn weight_to_fee(gas: u64, output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::ext_weight_to_fee(gas, output.as_ptr() as u32, output_len_ptr as u32)
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn set_rent_allowance(value: &[u8]) {
    unsafe { sys::ext_set_rent_allowance(value.as_ptr() as u32, value.len() as u32) }
}

pub fn random(subject: &[u8], output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::ext_random(
                subject.as_ptr() as u32,
                subject.len() as u32,
                output.as_ptr() as u32,
                output_len_ptr as u32,
            )
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn println(content: &str) {
    let bytes = content.as_bytes();
    unsafe { sys::ext_println(bytes.as_ptr() as u32, bytes.len() as u32) }
}

macro_rules! impl_hash_fn {
    ( $name:ident, $bytes_result:literal ) => {
        paste::item! {
            pub fn [<hash_ $name>](input: &[u8], output: &mut [u8; $bytes_result]) {
                unsafe {
                    sys::[<ext_hash_ $name>](
                        input.as_ptr() as u32,
                        input.len() as u32,
                        output.as_ptr() as u32,
                    )
                }
            }
        }
    };
}
impl_hash_fn!(sha2_256, 32);
impl_hash_fn!(keccak_256, 32);
impl_hash_fn!(blake2_256, 32);
impl_hash_fn!(blake2_128, 16);
