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

use pdsl_core::{
    env::{
        Address,
        Balance,
    },
    storage,
};
use pdsl_lang::contract;

contract! {
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
            self.total_supply.set(0);
            self.mint_for(env.caller(), init_value);
        }
    }

    impl Erc20 {
        /// Returns the total number of tokens in existence.
        pub(external) fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        /// Returns the balance of the given address.
        pub(external) fn balance_of(&self, owner: Address) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        /// Returns the amount of tokens that an owner allowed to a spender.
        pub(external) fn allowance(&self, owner: Address, spender: Address) -> Balance {
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }

        /// Transfers token from the sender to the `to` address.
        pub(external) fn transfer(&mut self, to: Address, value: Balance) -> bool {
            self.transfer_impl(env.caller(), to, value);
            true
        }

        /// Approve the passed address to spend the specified amount of tokens
        /// on the behalf of the message's sender.
        pub(external) fn approve(&mut self, spender: Address, value: Balance) -> bool {
            let owner = env.caller();
            self.allowances.insert((owner, spender), value);
            self.emit_approval(owner, spender, value);
            true
        }

        /// Transfer tokens from one address to another.
        pub(external) fn transfer_from(&mut self, from: Address, to: Address, value: Balance) -> bool {
            self.allowances[&(from, to)] -= value;
            self.transfer_impl(from, to, value);
            let new_allowance = self.allowances[&(from, to)];
            self.emit_transfer(from, to, value);
            self.emit_approval(from, to, new_allowance);
            true
        }
    }

    impl Erc20 {
        /// Transfers token from a specified address to another address.
        fn transfer_impl(&mut self, from: Address, to: Address, value: Balance) {
            self.balances[&from] -= value;
            self.balances[&to] += value;
            self.emit_transfer(from, to, value);
        }

        /// Decrease balance from the address.
        ///
        /// # Panics
        ///
        /// If `from` does not have enough balance.
        fn burn_for(&mut self, from: Address, value: Balance){
            self.balances[&from] -= value;
            self.total_supply -= value;
            self.emit_transfer(from, None, value);
        }

        /// Increase balance for the receiver out of nowhere.
        fn mint_for(&mut self, receiver: Address, value: Balance) {
            self.balances[&receiver] += value;
            self.total_supply += value;
            self.emit_transfer(None, receiver, value);
        }

        /// Emits an approval event.
        fn emit_approval(
            from: Address,
            to: Address,
            value: Balance,
        ) {
            assert!(from.is_some() && to.is_some());
            assert!(value > Balance::from(0));
            // emit event - This is not yet implemented in SRML contracts.
        }

        /// Emits a transfer event.
        fn emit_transfer<F, T>(
            from: F,
            to: T,
            value: Balance,
        )
        where
            F: Into<Option<Address>>,
            T: Into<Option<Address>>,
        {
            assert!(from.is_some() || to.is_some());
            assert!(value > Balance::from(0));
            // emit event - This is not yet implemented in SRML contracts.
        }
    }
}
