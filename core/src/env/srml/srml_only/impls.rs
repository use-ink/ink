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
        AccountId,
        Hash,
        srml::{
            sys,
            DefaultSrmlTypes,
        },
        Env,
        EnvStorage,
    },
    memory::vec::Vec,
    storage::Key,
};
use core::{
    marker::PhantomData,
};
use parity_codec::Decode;

/// The default SRML environment.
pub type DefaultSrmlEnv = SrmlEnv<DefaultSrmlTypes>;

/// The SRML contracts environment.
pub struct SrmlEnv<T>
{
    marker: PhantomData<T>,
}

impl<T> EnvStorage for SrmlEnv<T>
{
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
        let result = sys::ext_get_storage(key.as_bytes().as_ptr() as u32);
        if result != SUCCESS {
            return None
        }
        let size = sys::ext_scratch_size();
        let mut value = Vec::new();
        if size > 0 {
            value.resize(size as usize, 0);
            sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
        }
        Some(value)
    }
}

impl<T> Env for SrmlEnv<T>
{
    fn caller() -> AccountId {
        unsafe { sys::ext_caller() };
        let size = unsafe { sys::ext_scratch_size() };
        let mut value = Vec::new();
        if size > 0 {
            value.resize(size as usize, 0);
            unsafe {
                sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
            }
        }
        AccountId::decode(&mut value.as_slice()).expect("caller should be a valid Address")
    }

    fn input() -> Vec<u8> {
        let size = unsafe { sys::ext_input_size() };
        if size == 0 {
            Vec::new()
        } else {
            let mut buffer = Vec::new();
            buffer.resize(size as usize, 0);
            unsafe {
                sys::ext_input_copy(buffer.as_mut_ptr() as u32, 0, size);
            }
            buffer
        }
    }

    fn random_seed() -> Hash {
        unsafe { sys::ext_random_seed() };
        let size = unsafe { sys::ext_scratch_size() };
        let mut value = Vec::new();
        if size > 0 {
            value.resize(size as usize, 0);
            unsafe {
                sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
            }
        }
        Hash::decode(&mut value.as_slice()).expect("random seed should be a valid Hash")
    }

    unsafe fn r#return(data: &[u8]) -> ! {
        sys::ext_return(data.as_ptr() as u32, data.len() as u32);
    }

    fn println(content: &str) {
        unsafe { sys::ext_println(content.as_ptr() as u32, content.len() as u32) }
    }

    fn deposit_raw_event(data: &[u8]) {
        unsafe { sys::ext_deposit_event(data.as_ptr() as u32, data.len() as u32) }
    }
}
