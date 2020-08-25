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

        /// Every error that can be returned to a contract when it calls any of the host functions.
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
    /// Can only be returned from `seal_call` and `seal_instantiate`.
    CalleeTrapped = 1,
    /// The called function ran to completion but decided to revert its state.
    /// An output buffer is returned when one was supplied.
    /// Can only be returned from `seal_call` and `seal_instantiate`.
    CalleeReverted = 2,
    /// The passed key does not exist in storage.
    KeyNotFound = 3,
    /// Transfer failed because it would have brought the sender's total balance
    /// bwlow the subsistence threshold.
    BelowSubsistenceThreshold = 4,
    /// Transfer failed for other not further specified reason. Most probably
    /// reserved or locked balance of the sender that was preventing the transfer.
    TransferFailed = 5,
    /// The newly created contract is below the subsistence threshold after executing
    /// its constructor so no usable contract instance will be created.
    NewContractNotFunded = 6,
    /// No code could be found at the supplied code hash.
    CodeNotFound = 7,
    /// The account that was called is either no contract (e.g. user account) or is a tombstone.
    NotCallable = 8,
}

type Result = core::result::Result<(), Error>;

mod sys {
    use super::ReturnCode;

    #[link(wasm_import_module = "seal0")]
    extern "C" {
        pub fn seal_instantiate(
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
        ) -> ReturnCode;

        pub fn seal_call(
            callee_ptr: u32,
            callee_len: u32,
            gas: u64,
            transferred_value_ptr: u32,
            transferred_value_len: u32,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> ReturnCode;

        pub fn seal_transfer(
            account_id_ptr: u32,
            account_id_len: u32,
            transferred_value_ptr: u32,
            transferred_value_len: u32,
        ) -> ReturnCode;

        pub fn seal_deposit_event(
            topics_ptr: u32,
            topics_len: u32,
            data_ptr: u32,
            data_len: u32,
        );

        pub fn seal_set_storage(key_ptr: u32, value_ptr: u32, value_len: u32);
        pub fn seal_get_storage(key_ptr: u32, output_ptr: u32, output_len_ptr: u32)
            -> ReturnCode;
        pub fn seal_clear_storage(key_ptr: u32);

        pub fn seal_restore_to(
            dest_ptr: u32,
            dest_len: u32,
            code_hash_ptr: u32,
            code_hash_len: u32,
            rent_allowance_ptr: u32,
            rent_allowance_len: u32,
            delta_ptr: u32,
            delta_count: u32,
        );
        pub fn seal_terminate(beneficiary_ptr: u32, beneficiary_len: u32) -> !;

        pub fn seal_call_chain_extension(
            func_id: u32,
            input_ptr: u32,
            input_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        ) -> ReturnCode;

        pub fn seal_input(buf_ptr: u32, buf_len_ptr: u32);
        pub fn seal_return(flags: u32, data_ptr: u32, data_len: u32) -> !;

        pub fn seal_caller(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_block_number(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_address(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_balance(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_weight_to_fee(gas: u64, output_ptr: u32, output_len_ptr: u32);
        pub fn seal_gas_left(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_value_transferred(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_now(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_rent_allowance(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_minimum_balance(output_ptr: u32, output_len_ptr: u32);
        pub fn seal_tombstone_deposit(output_ptr: u32, output_len_ptr: u32);

        pub fn seal_set_rent_allowance(value_ptr: u32, value_len: u32);

        pub fn seal_random(
            subject_ptr: u32,
            subject_len: u32,
            output_ptr: u32,
            output_len_ptr: u32,
        );
        pub fn seal_println(str_ptr: u32, str_len: u32);

        pub fn seal_hash_keccak_256(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn seal_hash_blake2_256(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn seal_hash_blake2_128(input_ptr: u32, input_len: u32, output_ptr: u32);
        pub fn seal_hash_sha2_256(input_ptr: u32, input_len: u32, output_ptr: u32);
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
            sys::seal_instantiate(
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
    ret_code.into()
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
            sys::seal_call(
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
    ret_code.into()
}

pub fn transfer(account_id: &[u8], value: &[u8]) -> Result {
    let ret_code = unsafe {
        sys::seal_transfer(
            account_id.as_ptr() as u32,
            account_id.len() as u32,
            value.as_ptr() as u32,
            value.len() as u32,
        )
    };
    ret_code.into()
}

pub fn deposit_event(topics: &[u8], data: &[u8]) {
    unsafe {
        sys::seal_deposit_event(
            topics.as_ptr() as u32,
            topics.len() as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    }
}

pub fn set_storage(key: &[u8], encoded_value: &[u8]) {
    unsafe {
        sys::seal_set_storage(
            key.as_ptr() as u32,
            encoded_value.as_ptr() as u32,
            encoded_value.len() as u32,
        )
    }
}

pub fn clear_storage(key: &[u8]) {
    unsafe { sys::seal_clear_storage(key.as_ptr() as u32) }
}

pub fn get_storage(key: &[u8], output: &mut &mut [u8]) -> Result {
    let mut output_len = output.len() as u32;
    let ret_code = {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::seal_get_storage(
                key.as_ptr() as u32,
                output.as_ptr() as u32,
                output_len_ptr as u32,
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ret_code.into()
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
        sys::seal_restore_to(
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
    unsafe { sys::seal_terminate(beneficiary.as_ptr() as u32, beneficiary.len() as u32) }
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
            sys::seal_call_chain_extension(
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
        unsafe { sys::seal_input(output.as_ptr() as u32, output_len_ptr as u32) };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
    unsafe {
        sys::seal_return(
            flags.into_u32(),
            return_value.as_ptr() as u32,
            return_value.len() as u32,
        )
    }
}

macro_rules! impl_seal_wrapper_for {
    ( $( ($name:ident => $seal_name:ident), )* ) => {
        $(
            pub fn $name(output: &mut &mut [u8]) {
                let mut output_len = output.len() as u32;
                {
                    let output_len_ptr: *mut u32 = &mut output_len;
                    unsafe {
                        sys::$seal_name(
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
impl_seal_wrapper_for! {
    (caller => seal_caller),
    (block_number => seal_block_number),
    (address => seal_address),
    (balance => seal_balance),
    (gas_left => seal_gas_left),
    (value_transferred => seal_value_transferred),
    (now => seal_now),
    (rent_allowance => seal_rent_allowance),
    (minimum_balance => seal_minimum_balance),
    (tombstone_deposit => seal_tombstone_deposit),
}

pub fn weight_to_fee(gas: u64, output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::seal_weight_to_fee(gas, output.as_ptr() as u32, output_len_ptr as u32)
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn set_rent_allowance(value: &[u8]) {
    unsafe { sys::seal_set_rent_allowance(value.as_ptr() as u32, value.len() as u32) }
}

pub fn random(subject: &[u8], output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        let output_len_ptr: *mut u32 = &mut output_len;
        unsafe {
            sys::seal_random(
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
    unsafe { sys::seal_println(bytes.as_ptr() as u32, bytes.len() as u32) }
}

macro_rules! impl_hash_fn {
    ( $name:ident, $bytes_result:literal ) => {
        paste::item! {
            pub fn [<hash_ $name>](input: &[u8], output: &mut [u8; $bytes_result]) {
                unsafe {
                    sys::[<seal_hash_ $name>](
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
