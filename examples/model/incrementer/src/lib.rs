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

use pdsl_core::storage;
use pdsl_model::{
    messages,
    state,
    Contract,
    ContractDecl,
};

state! {
    /// A simple contract having just one value that can be incremented and returned.
    struct Adder {
        /// The simple value on the contract storage.
        val: storage::Value<u32>
    }
}

messages! {
    /// Increases the storage value by the given amount.
    0 => Inc(by: u32);
    /// Returns the storage value.
    1 => Get() -> u32;
}

#[rustfmt::skip]
fn instantiate() -> impl Contract {
	ContractDecl::using::<Adder>()
		.on_deploy(|env, init_val| {
			env.state.val.set(init_val)
		})
		.on_msg_mut::<Inc>(|env, by| {
			env.state.val += by
		})
		.on_msg::<Get>(|env, _| {
			*env.state.val.get()
		})
		.instantiate()
}

#[no_mangle]
fn deploy() {
    instantiate().deploy()
}

#[no_mangle]
fn call() {
    instantiate().dispatch()
}
