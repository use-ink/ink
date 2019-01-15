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

//! External C API to communicate with substrate contracts runtime module.
//!
//! Refer to substrate SRML contract module for more documentation.

extern "C" {
	// Currently unused due to rework of SRML contracts internals. (PUT_CODE)
	#[allow(unused)]
	pub fn ext_create(
		init_code_ptr: u32,
		init_code_len: u32,
		gas: u64,
		value_ptr: u32,
		value_len: u32,
		input_data_ptr: u32,
		input_data_len: u32
	) -> u32;

	// Currently unused due to rework of SRML contracts internals. (PUT_CODE)
	#[allow(unused)]
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
