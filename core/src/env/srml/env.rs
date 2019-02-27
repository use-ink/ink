// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    env::{
        srml::{
            self,
            sys,
        },
        Env,
        EnvTypes,
    },
    memory::vec::Vec,
    storage::Key,
};
use core::marker::PhantomData;

/// The SRML fundamental types.
pub struct DefaultSrmlTypes;

impl EnvTypes for DefaultSrmlTypes {
    type Address = srml::Address;
    type Balance = srml::Balance;
}

/// The default SRML environment.
pub type DefaultSrmlEnv = SrmlEnv<DefaultSrmlTypes>;

/// The SRML contracts environment.
pub struct SrmlEnv<T>
where
    T: EnvTypes,
{
    marker: PhantomData<T>,
}

impl<T> EnvTypes for SrmlEnv<T>
where
    T: EnvTypes,
{
    type Address = <T as EnvTypes>::Address;
    type Balance = <T as EnvTypes>::Balance;
}

impl<T> Env for SrmlEnv<T>
where
    T: EnvTypes,
    <T as EnvTypes>::Address: for<'a> From<&'a [u8]>,
{
    fn caller() -> <Self as EnvTypes>::Address {
        unsafe { sys::ext_caller() };
        let size = unsafe { sys::ext_scratch_size() };
        let mut value = Vec::new();
        if size > 0 {
            value.resize(size as usize, 0);
            unsafe {
                sys::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
            }
        }
        value.as_slice().into()
    }

    unsafe fn store(key: Key, value: &[u8]) {
        sys::ext_set_storage(
            key.as_bytes().as_ptr() as u32,
            1,
            value.as_ptr() as u32,
            value.len() as u32,
        );
    }

    unsafe fn clear(key: Key) {
        sys::ext_set_storage(key.as_bytes().as_ptr() as u32, 0, 0, 0)
    }

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

    fn return_(data: &[u8]) -> ! {
        unsafe {
            sys::ext_return(data.as_ptr() as u32, data.len() as u32);
        }
    }
}
