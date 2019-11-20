// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use crate::{
    env::{
        srml::sys,
        CallError,
        CreateError,
        Env,
        EnvStorage,
        EnvTypes,
    },
    memory::vec::Vec,
    storage::Key,
};
use core::marker::PhantomData;
use scale::{
    Decode,
    Encode,
};

/// Load the contents of the scratch buffer
fn read_scratch_buffer() -> Vec<u8> {
    let size = unsafe { sys::ext_scratch_size() };
    let mut value = Vec::new();
    if size > 0 {
        value.resize(size as usize, 0);
        unsafe {
            sys::ext_scratch_read(value.as_mut_ptr() as u32, 0, size);
        }
    }
    value
}

/// Writes the contents of `data` into the scratch buffer.
fn write_scratch_buffer(data: &[u8]) {
    unsafe {
        sys::ext_scratch_write(data.as_ptr() as u32, data.len() as u32);
    }
}

/// The SRML contract environment storage
pub enum SrmlEnvStorage {}

impl EnvStorage for SrmlEnvStorage {
    /// Stores the given bytes under the given key.
    unsafe fn store(key: Key, value: &[u8]) {
        sys::ext_set_storage(
            key.as_bytes().as_ptr() as u32,
            1,
            value.as_ptr() as u32,
            value.len() as u32,
        );
    }

    /// Clears the value stored under the given key.
    unsafe fn clear(key: Key) {
        sys::ext_set_storage(key.as_bytes().as_ptr() as u32, 0, 0, 0)
    }

    /// Loads the value stored at the given key if any.
    unsafe fn load(key: Key) -> Option<Vec<u8>> {
        const SUCCESS: u32 = 0;
        if sys::ext_get_storage(key.as_bytes().as_ptr() as u32) == SUCCESS {
            return Some(read_scratch_buffer())
        }
        None
    }
}

/// The SRML contracts environment.
pub struct SrmlEnv<T>
where
    T: EnvTypes,
{
    marker: PhantomData<fn() -> T>,
}

impl<T> EnvTypes for SrmlEnv<T>
where
    T: EnvTypes,
{
    type AccountId = <T as EnvTypes>::AccountId;
    type Balance = <T as EnvTypes>::Balance;
    type Hash = <T as EnvTypes>::Hash;
    type Moment = <T as EnvTypes>::Moment;
    type BlockNumber = <T as EnvTypes>::BlockNumber;
    type Call = <T as EnvTypes>::Call;
}

macro_rules! impl_getters_for_srml_env {
    ( $( ($name:ident, $ext_name:ident, $ret_type:ty) ),* ) => {
        $(
            fn $name() -> $ret_type {
                unsafe { sys::$ext_name() };
                Decode::decode(&mut &read_scratch_buffer()[..])
                    .expect(concat!(
                        stringify!($name), " expects to receive a correctly sized buffer"
                    ))
            }
        )*
    }
}

impl<T> SrmlEnv<T>
where
    T: EnvTypes,
{
    fn call(
        callee: <Self as EnvTypes>::AccountId,
        gas: u64,
        value: <Self as EnvTypes>::Balance,
        input_data: &[u8],
    ) -> u32 {
        let callee = callee.encode();
        let value = value.encode();
        unsafe {
            sys::ext_call(
                callee.as_ptr() as u32,
                callee.len() as u32,
                gas,
                value.as_ptr() as u32,
                value.len() as u32,
                input_data.as_ptr() as u32,
                input_data.len() as u32,
            )
        }
    }
}

impl<T> Env for SrmlEnv<T>
where
    T: EnvTypes,
{
    fn input() -> Vec<u8> {
        read_scratch_buffer()
    }

    impl_getters_for_srml_env!(
        (address, ext_address, <Self as EnvTypes>::AccountId),
        (balance, ext_balance, <Self as EnvTypes>::Balance),
        (caller, ext_caller, <Self as EnvTypes>::AccountId),
        (random_seed, ext_random_seed, <Self as EnvTypes>::Hash),
        (now, ext_now, <Self as EnvTypes>::Moment),
        (
            block_number,
            ext_block_number,
            <Self as EnvTypes>::BlockNumber
        ),
        (gas_price, ext_gas_price, <Self as EnvTypes>::Balance),
        (gas_left, ext_gas_left, <Self as EnvTypes>::Balance),
        (
            value_transferred,
            ext_value_transferred,
            <Self as EnvTypes>::Balance
        )
    );

    fn return_data(data: &[u8]) {
        write_scratch_buffer(data)
    }

    fn println(content: &str) {
        unsafe { sys::ext_println(content.as_ptr() as u32, content.len() as u32) }
    }

    fn deposit_raw_event(topics: &[<Self as EnvTypes>::Hash], data: &[u8]) {
        unsafe {
            sys::ext_deposit_event(
                topics.as_ptr() as u32,
                topics.len() as u32,
                data.as_ptr() as u32,
                data.len() as u32,
            )
        }
    }

    fn dispatch_raw_call(data: &[u8]) {
        unsafe { sys::ext_dispatch_call(data.as_ptr() as u32, data.len() as u32) }
    }

    fn call_invoke(
        callee: <Self as EnvTypes>::AccountId,
        gas: u64,
        value: <Self as EnvTypes>::Balance,
        input_data: &[u8],
    ) -> Result<(), CallError> {
        let result = Self::call(callee, gas, value, input_data);
        if result != 0 {
            return Err(CallError)
        }
        Ok(())
    }

    fn call_evaluate<U: Decode>(
        callee: <Self as EnvTypes>::AccountId,
        gas: u64,
        value: <Self as EnvTypes>::Balance,
        input_data: &[u8],
    ) -> Result<U, CallError> {
        let result = Self::call(callee, gas, value, input_data);
        if result != 0 {
            return Err(CallError)
        }
        U::decode(&mut &read_scratch_buffer()[..]).map_err(|_| CallError)
    }

    fn create(
        code_hash: <Self as EnvTypes>::Hash,
        gas_limit: u64,
        value: <Self as EnvTypes>::Balance,
        input_data: &[u8],
    ) -> Result<<Self as EnvTypes>::AccountId, CreateError> {
        let result = {
            let code_hash = code_hash.encode();
            let value = value.encode();
            unsafe {
                sys::ext_create(
                    code_hash.as_ptr() as u32,
                    code_hash.len() as u32,
                    gas_limit,
                    value.as_ptr() as u32,
                    value.len() as u32,
                    input_data.as_ptr() as u32,
                    input_data.len() as u32,
                )
            }
        };
        if result != 0 {
            return Err(CreateError)
        }
        <Self as EnvTypes>::AccountId::decode(&mut &read_scratch_buffer()[..])
            .map_err(|_| CreateError)
    }
}
