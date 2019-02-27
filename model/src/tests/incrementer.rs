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
	ContractDecl,
	TestableContract,
	state,
	messages,
};
use pdsl_core::storage;

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

fn instantiate() -> impl TestableContract<DeployArgs = u32> {
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

#[test]
fn inc_and_read() {
	let mut contract = instantiate();
	contract.deploy(0_u32);
	assert_eq!(contract.call::<Get>(()), 0_u32);
	contract.call::<Inc>(1);
	assert_eq!(contract.call::<Get>(()), 1_u32);
	contract.call::<Inc>(41);
	assert_eq!(contract.call::<Get>(()), 42_u32);
}

#[test]
#[should_panic]
fn read_without_deploy() {
	let mut contract = instantiate();
	let _res = contract.call::<Get>(());
}

#[test]
#[should_panic]
fn write_without_deploy() {
	let mut contract = instantiate();
	contract.call::<Inc>(100);
}
