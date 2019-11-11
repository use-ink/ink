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

#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{
	env::DefaultSrmlTypes,
	memory::format,
	storage,
};
use ink_lang::contract;
use ink_model::EnvHandler;

contract! {
	#![env = DefaultSrmlTypes]

	event Transfer {
		from: Option<AccountId>,
		to: Option<AccountId>,
		value: Balance,
	}

	event Approval {
		owner: AccountId,
		spender: AccountId,
		value: Balance,
	}

	struct Erc20 {
		total_supply: storage::Value<Balance>,
		balances: storage::HashMap<AccountId, Balance>,
		allowances: storage::HashMap<(AccountId, AccountId), Balance>,
	}

	impl Deploy for Erc20 {
		fn deploy(&mut self, initial_supply: Balance) {
			let caller = env.caller();
			self.total_supply.set(initial_supply);
			self.balances.insert(caller, initial_supply);
			env.emit(Transfer {
				from: None,
				to: Some(caller),
				value: initial_supply,
			});
		}
	}

	impl Erc20 {

		pub(external) fn total_supply(&self) -> Balance {
			*self.total_supply
		}

		pub(external) fn balance_of(&self, owner: AccountId) -> Balance {
			self.balance_of_or_zero(&owner)
		}

		pub(external) fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
			self.allowance_of_or_zero(&owner, &spender)
		}

		pub(external) fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
			let from = env.caller();
			self.transfer_from_to(env, from, to, value)
		}

		pub(external) fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
			let owner = env.caller();
			self.allowances.insert((owner, spender), value);
			env.emit(Approval {
				owner,
				spender,
				value,
			});
			true
		}

		pub(external) fn transfer_from(
			&mut self,
			from: AccountId,
			to: AccountId,
			value: Balance,
		) -> bool {
			let caller = env.caller();
			let allowance = self.allowance_of_or_zero(&from, &caller);
			if allowance < value {
				return false;
			}
			self.allowances.insert((from, caller), allowance - value);
			self.transfer_from_to(env, from, to, value)
		}

		fn transfer_from_to(
			&mut self,
			env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>,
			from: AccountId,
			to: AccountId,
			value: Balance,
		) -> bool {
			let from_balance = self.balance_of_or_zero(&from);
			if from_balance < value {
				return false
			}
			let to_balance = self.balance_of_or_zero(&to);
			self.balances.insert(from, from_balance - value);
			self.balances.insert(to, to_balance + value);
			env.emit(Transfer {
				from: Some(from),
				to: Some(to),
				value
			});
			true
		}

		fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
			*self.balances.get(owner).unwrap_or(&0)
		}

		fn allowance_of_or_zero(
			&self,
			owner: &AccountId,
			spender: &AccountId,
		) -> Balance {
			*self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
		}
	}
}
