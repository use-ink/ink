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

use ink_core::{
    env::{
        Address,
        Balance,
    },
    storage,
};
use ink_model::{
    messages,
    state,
    Contract,
    ContractDecl,
};

state! {
    /// A simple implementation of a rudimentary Erc20 token contract.
    struct Erc20Token {
        /// The balance for an address.
        balances: storage::HashMap<Address, Balance>,
        /// The total supply.
        total: storage::Value<Balance>
    }
}

messages! {
    /// Returns the total supply.
    0 => TotalSupply() -> Balance;
    /// Returns the balance of the given address.
    1 => BalanceOf(owner: Address) -> Balance;
    /// Transfers balance from the caller to the given address.
    ///
    /// Returns `true` if the transfer was successful.
    2 => Transfer(to: Address, amount: Balance) -> bool;
}

#[rustfmt::skip]
fn instantiate() -> impl Contract {
	ContractDecl::using::<Erc20Token>()
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
