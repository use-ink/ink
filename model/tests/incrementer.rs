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

use ink_core::{
    env::{
        ContractEnv,
        DefaultSrmlTypes,
    },
    storage,
};
use ink_model::{
    messages,
    state,
    ContractDecl,
    TestableContract,
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
    [0u8; 4] => Inc(by: u32);
    /// Returns the storage value.
    [1, 0, 0, 0] => Get() -> u32;
}

#[rustfmt::skip]
fn instantiate() -> impl TestableContract<DeployArgs = u32> {
	ContractDecl::using::<Adder, ContractEnv<DefaultSrmlTypes>>()
		.on_deploy(|env, init_val| {
			env.state.val.set(init_val)
		})
		.on_msg_mut::<Inc>(|env, by| {
			env.state.val += by
		})
		.on_msg::<Get>(|env, _| {
			*env.state.val
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
