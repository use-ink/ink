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

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use pdsl_core::{
    storage,
    env::{Address, Balance, deposit_raw_event, println},
    memory::format
};
use pdsl_lang::contract;

contract! {
    /// The storage items for a typical ERC20 token implementation.
    struct Erc20 {
        /// All peeps done by all users.
        balances: storage::HashMap<Address, Balance>,
        /// Balances that are spendable by non-owners.
        ///
        /// # Note
        ///
        /// Mapping: (from, to) -> allowed
        allowances: storage::HashMap<(Address, Address), Balance>,
        /// The total supply.
        total_supply: storage::Value<Balance>,
    }

    impl Deploy for Erc20 {
        fn deploy(&mut self, init_value: Balance) {
            // We have to set total supply to `0` in order to initialize it.
            // Otherwise accesses to total supply will panic.
            self.total_supply.set(init_value);
            self.balances.insert(env.caller(), init_value);
            Self::emit_transfer(None, env.caller(), init_value);
        }
    }

    impl Erc20 {
        /// Returns the total number of tokens in existence.
        pub(external) fn total_supply(&self) -> Balance {
            println(&format!("Total Supply: {:?}", *self.total_supply));
            *self.total_supply
        }

        /// Returns the balance of the given address.
        pub(external) fn balance_of(&self, owner: Address) -> Balance {
            let balance = *self.balances.get(&owner).unwrap_or(&0);
            println(&format!("Balance of {:?}: {:?}", owner, balance));
            balance
        }

        /// Returns the amount of tokens that an owner allowed to a spender.
        pub(external) fn allowance(&self, owner: Address, spender: Address) -> Balance {
            let allowance = *self.allowances.get(&(owner, spender)).unwrap_or(&0);
            println(&format!("Allowance: {:?} is allowed to spend {:?} from {:?}", spender, allowance, owner));
            allowance
        }

        /// Transfers token from the sender to the `to` address.
        pub(external) fn transfer(&mut self, to: Address, value: Balance) -> bool {
            let owner = env.caller();

            let balance_owner = *self.balances.get(&owner).unwrap_or(&0);
            let balance_to = *self.balances.get(&to).unwrap_or(&0);

            let new_balance_owner = balance_owner.checked_sub(value).unwrap();
            let new_balance_to = balance_to.checked_add(value).unwrap();

            self.balances.insert(owner, new_balance_owner);
            self.balances.insert(to, new_balance_to);
            
            Self::emit_transfer(owner, to, value);

            true
        }

        /// Approve the passed address to spend the specified amount of tokens
        /// on the behalf of the message's sender.
        pub(external) fn approve(&mut self, spender: Address, value: Balance) -> bool {
            let owner = env.caller();

            self.allowances.insert((owner, spender), value);

            Self::emit_approval(owner, spender, value);

            true
        }

        /// Transfer tokens from one address to another.
        pub(external) fn transfer_from(&mut self, from: Address, to: Address, value: Balance) -> bool {
            let spender = env.caller();

            let allowance = *self.allowances.get(&(from, spender)).unwrap_or(&0);
            let balance_from = *self.balances.get(&from).unwrap_or(&0);
            let balance_to = *self.balances.get(&to).unwrap_or(&0);

            let new_allowance = allowance.checked_sub(value).unwrap();
            let new_balance_from = balance_from.checked_sub(value).unwrap();
            let new_balance_to = balance_to.checked_add(value).unwrap();

            self.allowances.insert((from, spender), new_allowance);
            self.balances.insert(from, new_balance_from);
            self.balances.insert(to, new_balance_to);

            Self::emit_transfer(from, to, value);
            
            true
        }
    }

    impl Erc20 {
        fn emit_transfer<F>(from: F, to: Address, value: Balance)
            where F: Into<Option<Address>>
        {
            deposit_raw_event(&[0x0]);
        }

        fn emit_approval(tokenOwner: Address, spender: Address, value: Balance) {
            deposit_raw_event(&[0x1]);
        }
    }
}