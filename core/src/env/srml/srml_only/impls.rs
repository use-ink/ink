// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    env::{
        srml::sys,
        Env,
        EnvStorage,
        EnvTypes,
    },
    memory::vec::Vec,
    storage::Key,
};
use core::marker::PhantomData;
use parity_codec::Decode;

/// Load the contents of the scratch buffer
fn read_scratch_buffer() -> Vec<u8> {
    let size = unsafe { sys::ext_scratch_size() };
    let mut value = Vec::new();
    if size > 0 {
        value.resize(size as usize, 0);
        unsafe {
            sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
        }
    }
    value
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
}

macro_rules! impl_getters_for_srml_env {
    ( $( ($name:ident, $ext_name:ident, $ret_type:ty) ),* ) => {
        $(
            fn $name() -> $ret_type {
                unsafe { sys::$ext_name() };
                Decode::decode(&mut &read_scratch_buffer()[..])
                    .ok_or(concat!(
                        stringify!($name), " received an incorrectly sized buffer from SRML"
                    ))
                    .expect(concat!(
                        stringify!($name), " expects to receive a correctly sized buffer"
                    ))
            }
        )*
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

    unsafe fn r#return(data: &[u8]) -> ! {
        sys::ext_return(data.as_ptr() as u32, data.len() as u32);
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
}
