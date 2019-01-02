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

use super::*;

/// Refer to substrate SRML contract module for more documentation.
pub mod c_abi {
	extern "C" {
		pub fn ext_create(
			init_code_ptr: u32,
			init_code_len: u32,
			gas: u64,
			value_ptr: u32,
			value_len: u32,
			input_data_ptr: u32,
			input_data_len: u32
		) -> u32;

		pub fn ext_call(
			callee_ptr: u32,
			callee_len: u32,
			gas: u64,
			value_ptr: u32,
			value_len: u32,
			input_data_ptr: u32,
			input_data_len: u32
		) -> u32;

		pub fn ext_caller();

		pub fn ext_set_storage(
			key_ptr: u32,
			value_non_null: u32,
			value_ptr: u32,
			value_len: u32
		);
		pub fn ext_get_storage(key_ptr: u32) -> u32;

		pub fn ext_scratch_size() -> u32;
		pub fn ext_scratch_copy(dest_ptr: u32, offset: u32, len: u32);

		pub fn ext_input_size() -> u32;
		pub fn ext_input_copy(dest_ptr: u32, offset: u32, len: u32);

		pub fn ext_return(data_ptr: u32, data_len: u32) -> !;
	}
}

/// The default SRML contracts environment.
pub struct SrmlEnv;

impl Env for SrmlEnv {
	fn caller() -> Vec<u8> {
		unsafe { c_abi::ext_caller() };
		let size = unsafe { c_abi::ext_scratch_size() };
		let mut value = Vec::new();
		if size > 0 {
			value.resize(size as usize, 0);
			unsafe {
				c_abi::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
			}
		}
		value
	}

	unsafe fn store(key: Key, value: &[u8]) {
		c_abi::ext_set_storage(
			key.as_bytes().as_ptr() as u32,
			1,
			value.as_ptr() as u32,
			value.len() as u32
		);
	}

	unsafe fn clear(key: Key) {
		c_abi::ext_set_storage(key.as_bytes().as_ptr() as u32, 0, 0, 0)
	}

	unsafe fn load(key: Key) -> Option<Vec<u8>> {
		const SUCCESS: u32 = 0;
		let result = c_abi::ext_get_storage(key.as_bytes().as_ptr() as u32);
		if result != SUCCESS {
			return None
		}
		let size = c_abi::ext_scratch_size();
		let mut value = Vec::new();
		if size > 0 {
			value.resize(size as usize, 0);
			c_abi::ext_scratch_copy(value.as_mut_ptr() as u32, 0, size);
		}
		Some(value)
	}

	fn input() -> Vec<u8> {
		let size = unsafe { c_abi::ext_input_size() };
		if size == 0 {
			Vec::new()
		} else {
			let mut buffer = Vec::new();
			buffer.resize(size as usize, 0);
			unsafe { c_abi::ext_input_copy(buffer.as_mut_ptr() as u32, 0, size); }
			buffer
		}
	}

	fn return_(data: &[u8]) -> ! {
		unsafe {
			c_abi::ext_return(data.as_ptr() as u32, data.len() as u32);
		}
	}
}
