// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use crate::ReturnFlags;
use core::marker::PhantomData;
use ink_primitives::Key;

macro_rules! define_error_codes {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident = $discr:literal,
        )*
    ) => {
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
                match return_code.0 {
                    0 => Ok(()),
                    $(
                        $discr => Err(Error::$name),
                    )*
                    _ => Err(Error::UnknownError),
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
    /// below the subsistence threshold.
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

/// Thin-wrapper around a `u32` representing a pointer for Wasm32.
///
/// Only for shared references.
///
/// # Note
///
/// Can only be constructed from shared reference types and encapsulates the
/// conversion from reference to raw `u32`.
/// Does not allow accessing the internal `u32` value.
#[derive(Debug)]
#[repr(transparent)]
pub struct Ptr32<'a, T>
where
    T: ?Sized,
{
    /// The internal Wasm32 raw pointer value.
    ///
    /// Must not be readable or directly usable by any safe Rust code.
    value: u32,
    /// We handle types like these as if the associated lifetime was exclusive.
    marker: PhantomData<fn() -> &'a T>,
}

impl<'a, T> Ptr32<'a, T>
where
    T: ?Sized,
{
    /// Creates a new Wasm32 pointer for the given raw pointer value.
    fn new(value: u32) -> Self {
        Self {
            value,
            marker: Default::default(),
        }
    }
}

impl<'a, T> Ptr32<'a, [T]> {
    /// Creates a new Wasm32 pointer from the given shared slice.
    pub fn from_slice(slice: &'a [T]) -> Self {
        Self::new(slice.as_ptr() as u32)
    }
}

/// Thin-wrapper around a `u32` representing a pointer for Wasm32.
///
/// Only for exclusive references.
///
/// # Note
///
/// Can only be constructed from exclusive reference types and encapsulates the
/// conversion from reference to raw `u32`.
/// Does not allow accessing the internal `u32` value.
#[derive(Debug)]
#[repr(transparent)]
pub struct Ptr32Mut<'a, T>
where
    T: ?Sized,
{
    /// The internal Wasm32 raw pointer value.
    ///
    /// Must not be readable or directly usable by any safe Rust code.
    value: u32,
    /// We handle types like these as if the associated lifetime was exclusive.
    marker: PhantomData<fn() -> &'a mut T>,
}

impl<'a, T> Ptr32Mut<'a, T>
where
    T: ?Sized,
{
    /// Creates a new Wasm32 pointer for the given raw pointer value.
    fn new(value: u32) -> Self {
        Self {
            value,
            marker: Default::default(),
        }
    }
}

impl<'a, T> Ptr32Mut<'a, [T]> {
    /// Creates a new Wasm32 pointer from the given exclusive slice.
    pub fn from_slice(slice: &'a mut [T]) -> Self {
        Self::new(slice.as_ptr() as u32)
    }
}

impl<'a, T> Ptr32Mut<'a, T>
where
    T: Sized,
{
    /// Creates a new Wasm32 pointer from the given exclusive reference.
    pub fn from_ref(a_ref: &'a mut T) -> Self {
        let a_ptr: *mut T = a_ref;
        Self::new(a_ptr as u32)
    }
}

/// The raw return code returned by the host side.
#[repr(transparent)]
pub struct ReturnCode(u32);

impl ReturnCode {
    /// Returns the raw underlying `u32` representation.
    pub fn into_u32(self) -> u32 {
        self.0
    }
}

type Result = core::result::Result<(), Error>;

mod sys {
    use super::{
        Key,
        Ptr32,
        Ptr32Mut,
        ReturnCode,
    };

    #[link(wasm_import_module = "seal0")]
    extern "C" {
        pub fn seal_instantiate(
            init_code_ptr: Ptr32<[u8]>,
            init_code_len: u32,
            gas: u64,
            endowment_ptr: Ptr32<[u8]>,
            endowment_len: u32,
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            address_ptr: Ptr32Mut<[u8]>,
            address_len_ptr: Ptr32Mut<u32>,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
            salt_ptr: Ptr32<[u8]>,
            salt_len: u32,
        ) -> ReturnCode;

        pub fn seal_call(
            callee_ptr: Ptr32<[u8]>,
            callee_len: u32,
            gas: u64,
            transferred_value_ptr: Ptr32<[u8]>,
            transferred_value_len: u32,
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        ) -> ReturnCode;

        pub fn seal_transfer(
            account_id_ptr: Ptr32<[u8]>,
            account_id_len: u32,
            transferred_value_ptr: Ptr32<[u8]>,
            transferred_value_len: u32,
        ) -> ReturnCode;

        pub fn seal_deposit_event(
            topics_ptr: Ptr32<[u8]>,
            topics_len: u32,
            data_ptr: Ptr32<[u8]>,
            data_len: u32,
        );

        pub fn seal_set_storage(
            key_ptr: Ptr32<[u8]>,
            value_ptr: Ptr32<[u8]>,
            value_len: u32,
        );
        pub fn seal_get_storage(
            key_ptr: Ptr32<[u8]>,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        ) -> ReturnCode;
        pub fn seal_clear_storage(key_ptr: Ptr32<[u8]>);

        pub fn seal_restore_to(
            dest_ptr: Ptr32<[u8]>,
            dest_len: u32,
            code_hash_ptr: Ptr32<[u8]>,
            code_hash_len: u32,
            rent_allowance_ptr: Ptr32<[u8]>,
            rent_allowance_len: u32,
            delta_ptr: Ptr32<[Key]>,
            delta_count: u32,
        );
        pub fn seal_terminate(beneficiary_ptr: Ptr32<[u8]>, beneficiary_len: u32) -> !;

        pub fn seal_call_chain_extension(
            func_id: u32,
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        ) -> ReturnCode;

        pub fn seal_input(buf_ptr: Ptr32Mut<[u8]>, buf_len_ptr: Ptr32Mut<u32>);
        pub fn seal_return(flags: u32, data_ptr: Ptr32<[u8]>, data_len: u32) -> !;

        pub fn seal_caller(output_ptr: Ptr32Mut<[u8]>, output_len_ptr: Ptr32Mut<u32>);
        pub fn seal_block_number(
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_address(output_ptr: Ptr32Mut<[u8]>, output_len_ptr: Ptr32Mut<u32>);
        pub fn seal_balance(output_ptr: Ptr32Mut<[u8]>, output_len_ptr: Ptr32Mut<u32>);
        pub fn seal_weight_to_fee(
            gas: u64,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_gas_left(output_ptr: Ptr32Mut<[u8]>, output_len_ptr: Ptr32Mut<u32>);
        pub fn seal_value_transferred(
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_now(output_ptr: Ptr32Mut<[u8]>, output_len_ptr: Ptr32Mut<u32>);
        pub fn seal_rent_allowance(
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_minimum_balance(
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_tombstone_deposit(
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );

        pub fn seal_set_rent_allowance(value_ptr: Ptr32<[u8]>, value_len: u32);

        pub fn seal_random(
            subject_ptr: Ptr32<[u8]>,
            subject_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
            output_len_ptr: Ptr32Mut<u32>,
        );
        pub fn seal_println(str_ptr: Ptr32<[u8]>, str_len: u32);

        pub fn seal_hash_keccak_256(
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
        );
        pub fn seal_hash_blake2_256(
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
        );
        pub fn seal_hash_blake2_128(
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
        );
        pub fn seal_hash_sha2_256(
            input_ptr: Ptr32<[u8]>,
            input_len: u32,
            output_ptr: Ptr32Mut<[u8]>,
        );
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
    salt: &[u8],
) -> Result {
    let mut address_len = out_address.len() as u32;
    let mut return_value_len = out_return_value.len() as u32;
    let ret_code = {
        unsafe {
            sys::seal_instantiate(
                Ptr32::from_slice(code_hash),
                code_hash.len() as u32,
                gas_limit,
                Ptr32::from_slice(endowment),
                endowment.len() as u32,
                Ptr32::from_slice(input),
                input.len() as u32,
                Ptr32Mut::from_slice(out_address),
                Ptr32Mut::from_ref(&mut address_len),
                Ptr32Mut::from_slice(out_return_value),
                Ptr32Mut::from_ref(&mut return_value_len),
                Ptr32::from_slice(salt),
                salt.len() as u32,
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
        unsafe {
            sys::seal_call(
                Ptr32::from_slice(callee),
                callee.len() as u32,
                gas_limit,
                Ptr32::from_slice(value),
                value.len() as u32,
                Ptr32::from_slice(input),
                input.len() as u32,
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ret_code.into()
}

pub fn transfer(account_id: &[u8], value: &[u8]) -> Result {
    let ret_code = unsafe {
        sys::seal_transfer(
            Ptr32::from_slice(account_id),
            account_id.len() as u32,
            Ptr32::from_slice(value),
            value.len() as u32,
        )
    };
    ret_code.into()
}

pub fn deposit_event(topics: &[u8], data: &[u8]) {
    unsafe {
        sys::seal_deposit_event(
            Ptr32::from_slice(topics),
            topics.len() as u32,
            Ptr32::from_slice(data),
            data.len() as u32,
        )
    }
}

pub fn set_storage(key: &[u8], encoded_value: &[u8]) {
    unsafe {
        sys::seal_set_storage(
            Ptr32::from_slice(key),
            Ptr32::from_slice(encoded_value),
            encoded_value.len() as u32,
        )
    }
}

pub fn clear_storage(key: &[u8]) {
    unsafe { sys::seal_clear_storage(Ptr32::from_slice(key)) }
}

pub fn get_storage(key: &[u8], output: &mut &mut [u8]) -> Result {
    let mut output_len = output.len() as u32;
    let ret_code = {
        unsafe {
            sys::seal_get_storage(
                Ptr32::from_slice(key),
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
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
            Ptr32::from_slice(account_id),
            account_id.len() as u32,
            Ptr32::from_slice(code_hash),
            code_hash.len() as u32,
            Ptr32::from_slice(rent_allowance),
            rent_allowance.len() as u32,
            Ptr32::from_slice(filtered_keys),
            filtered_keys.len() as u32,
        )
    }
}

pub fn terminate(beneficiary: &[u8]) -> ! {
    unsafe {
        sys::seal_terminate(Ptr32::from_slice(beneficiary), beneficiary.len() as u32)
    }
}

pub fn call_chain_extension(func_id: u32, input: &[u8], output: &mut &mut [u8]) -> u32 {
    let mut output_len = output.len() as u32;
    let ret_code = {
        unsafe {
            sys::seal_call_chain_extension(
                func_id,
                Ptr32::from_slice(input),
                input.len() as u32,
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
            )
        }
    };
    extract_from_slice(output, output_len as usize);
    ret_code.into_u32()
}

pub fn input(output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        unsafe {
            sys::seal_input(
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
            )
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn return_value(flags: ReturnFlags, return_value: &[u8]) -> ! {
    unsafe {
        sys::seal_return(
            flags.into_u32(),
            Ptr32::from_slice(return_value),
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
                    unsafe {
                        sys::$seal_name(
                            Ptr32Mut::from_slice(output),
                            Ptr32Mut::from_ref(&mut output_len),
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
        unsafe {
            sys::seal_weight_to_fee(
                gas,
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
            )
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn set_rent_allowance(value: &[u8]) {
    unsafe { sys::seal_set_rent_allowance(Ptr32::from_slice(value), value.len() as u32) }
}

pub fn random(subject: &[u8], output: &mut &mut [u8]) {
    let mut output_len = output.len() as u32;
    {
        unsafe {
            sys::seal_random(
                Ptr32::from_slice(subject),
                subject.len() as u32,
                Ptr32Mut::from_slice(output),
                Ptr32Mut::from_ref(&mut output_len),
            )
        };
    }
    extract_from_slice(output, output_len as usize);
}

pub fn println(content: &str) {
    let bytes = content.as_bytes();
    unsafe { sys::seal_println(Ptr32::from_slice(bytes), bytes.len() as u32) }
}

macro_rules! impl_hash_fn {
    ( $name:ident, $bytes_result:literal ) => {
        paste::item! {
            pub fn [<hash_ $name>](input: &[u8], output: &mut [u8; $bytes_result]) {
                unsafe {
                    sys::[<seal_hash_ $name>](
                        Ptr32::from_slice(input),
                        input.len() as u32,
                        Ptr32Mut::from_slice(output),
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
