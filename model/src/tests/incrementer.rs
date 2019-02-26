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
	Contract,
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

fn instantiate() -> impl TestableContract {
	ContractDecl::using::<Adder>()
		.on_deploy(|env, init_val| {
			println!("Incrementer::on_deploy");
			env.state.val.set(init_val)
		})
		.on_msg_mut::<Inc>(|env, by| {
			println!("Incrementer::inc");
			env.state.val += by
		})
		.on_msg::<Get>(|env, _| {
			println!("Incrementer::get -> {:?}", *env.state.val.get());
			*env.state.val.get()
		})
		.instantiate()
}

#[test]
fn inc_and_read() {
	use pdsl_core::env::TestEnv;
	TestEnv::set_input(b"\0\0\0\0"); // 4 zero-bytes to initialize the `u32` with `0x0`
	instantiate().deploy();
	let mut contract = instantiate();
	assert_eq!(contract.call::<Get>(()), 0_u32);
	contract.call::<Inc>(1);
	assert_eq!(contract.call::<Get>(()), 1_u32);
	contract.call::<Inc>(41);
	assert_eq!(contract.call::<Get>(()), 42_u32);
}
