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

#![no_std]

// use ink_core::{
// 	env::{Env, ContractEnv},
// };

#[allow(unused)]
use ink_core;

#[no_mangle]
pub extern "C" fn deploy() {
    // ContractEnv::println("noop contract: CREATE");
}

#[no_mangle]
pub extern "C" fn call() {
    // ContractEnv::println("noop contract: CALL");
}
