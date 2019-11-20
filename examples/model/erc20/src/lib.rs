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

#![no_std]

use ink_core::{
    env::{
        ContractEnv,
        EnvTypes,
        DefaultSrmlTypes,
    },
    storage,
};
use ink_model::{
    messages,
    state,
    Contract,
    ContractDecl,
};

type AccountId = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::AccountId;
type Balance = <ContractEnv<DefaultSrmlTypes> as EnvTypes>::Balance;

state! {
    /// A simple implementation of a rudimentary Erc20 token contract.
    struct Erc20Token {
        /// The balance for an address.
        balances: storage::HashMap<AccountId, Balance>,
        /// The total supply.
        total: storage::Value<Balance>
    }
}

messages! {
    /// Returns the total supply.
    0 => TotalSupply() -> Balance;
    /// Returns the balance of the given address.
    1 => BalanceOf(owner: AccountId) -> Balance;
    /// Transfers balance from the caller to the given address.
    ///
    /// Returns `true` if the transfer was successful.
    2 => Transfer(to: AccountId, amount: Balance) -> bool;
}

#[rustfmt::skip]
fn instantiate() -> impl Contract {
	ContractDecl::using::<Erc20Token, ContractEnv<DefaultSrmlTypes>>()
		.on_deploy(|env, init_supply| {
			let caller = env.caller();
			env.state.balances[&caller] = init_supply;
			env.state.total.set(init_supply);
		})
		.on_msg::<TotalSupply>(|env, _| {
			*env.state.total.get()
		})
		.on_msg::<BalanceOf>(|env, owner| {
			env.state.balances[&owner]
		})
		.on_msg_mut::<Transfer>(|env, (to, amount)| {
			let from = env.caller();
			let balance_from = env.state.balances[&from];
			let balance_to = env.state.balances[&to];

			if balance_from >= amount {
				env.state.balances[&from] = balance_from - amount;
				env.state.balances[&to] = balance_to + amount;
				return true
			}

			false
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
