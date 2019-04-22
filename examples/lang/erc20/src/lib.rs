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

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use parity_codec::{
    Decode,
    Encode,
};
use pdsl_core::{
    env::{
        self,
        Address,
        Balance,
    },
    memory::format,
    storage,
};
use pdsl_lang::contract;

/// Events deposited by the ERC20 token contract.
#[derive(Encode, Decode)]
enum Event {
    /// An approval for allowance was made.
    Approval {
        from: Address,
        to: Address,
        value: Balance,
    },
    /// A transfer has been done.
    Transfer {
        from: Option<Address>,
        to: Option<Address>,
        value: Balance,
    },
}

/// Deposits an ERC20 token event.
fn deposit_event(event: Event) {
    env::deposit_raw_event(&event.encode()[..])
}

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
            env.println(&format!("Erc20::deploy(caller = {:?}, init_value = {:?})", env.caller(), init_value));
            self.total_supply.set(0);
            self.mint_for(env.caller(), init_value);
        }
    }

    impl Erc20 {
        /// Returns the total number of tokens in existence.
        pub(external) fn total_supply(&self) -> Balance {
            let total_supply = *self.total_supply;
            env.println(&format!("Erc20::total_supply = {:?}", total_supply));
            total_supply
        }

        /// Returns the balance of the given address.
        pub(external) fn balance_of(&self, owner: Address) -> Balance {
            let balance = *self.balances.get(&owner).unwrap_or(&0);
            env.println(&format!("Erc20::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// Returns the amount of tokens that an owner allowed to a spender.
        pub(external) fn allowance(&self, owner: Address, spender: Address) -> Balance {
            self.allowance_or_zero(&owner, &spender)
        }

        /// Transfers token from the sender to the `to` address.
        pub(external) fn transfer(&mut self, to: Address, value: Balance) -> bool {
            env.println(&format!(
                "Erc20::transfer(to = {:?}, value = {:?})",
                to, value
            ));
            self.transfer_impl(env.caller(), to, value)
        }

        /// Approve the passed address to spend the specified amount of tokens
        /// on the behalf of the message's sender.
        pub(external) fn approve(&mut self, spender: Address, value: Balance) -> bool {
            env.println(&format!(
                "Erc20::approve(spender = {:?}, value = {:?})",
                spender, value
            ));
            let owner = env.caller();
            if owner == spender || value == 0 {
                return false
            }
            self.allowances.insert((owner, spender), value);
            Self::emit_approval(owner, spender, value);
            true
        }

        /// Transfer tokens from one address to another.
        pub(external) fn transfer_from(&mut self, from: Address, to: Address, value: Balance) -> bool {
            env.println(&format!(
                "Erc20::transfer_from(from: {:?}, to = {:?}, value = {:?})",
                from, to, value
            ));
            self.transfer_impl(from, to, value)
        }
    }

    impl Erc20 {
        /// Decreases the allowance and returns if it was successful.
        fn try_decrease_allowance(&mut self, from: &Address, to: &Address, by: Balance) -> bool {
            // The owner of the coins doesn't need an allowance.
            if &env::caller() == from {
                return true
            }
            let allowance = self.allowance_or_zero(from, to);
            if allowance < by {
                return false
            }
            self.allowances.insert((*from, *to), allowance - by);
            true
        }

        /// Returns the allowance or 0 of there is no allowance.
        fn allowance_or_zero(&self, from: &Address, to: &Address) -> Balance {
            let allowance = self.allowances.get(&(*from, *to)).unwrap_or(&0);
            env::println(&format!(
                "Erc20::allowance_or_zero(from = {:?}, to = {:?}) = {:?}",
                from, to, allowance
            ));
            *allowance
        }

        /// Returns the balance of the address or 0 if there is no balance.
        fn balance_of_or_zero(&self, of: &Address) -> Balance {
            let balance = self.balances.get(of).unwrap_or(&0);
            env::println(&format!(
                "Erc20::balance_of_or_zero(of = {:?}) = {:?}",
                of, balance
            ));
            *balance
        }

        /// Transfers token from a specified address to another address.
        fn transfer_impl(&mut self, from: Address, to: Address, value: Balance) -> bool {
            env::println(&format!(
                "Erc20::transfer_impl(from = {:?}, to = {:?}, value = {:?})",
                from, to, value
            ));
            let balance_from = self.balance_of_or_zero(&from);
            let balance_to = self.balance_of_or_zero(&to);
            if balance_from < value {
                return false
            }
            if !self.try_decrease_allowance(&from, &to, value) {
                return false
            }
            self.balances.insert(from, balance_from - value);
            self.balances.insert(to, balance_to + value);
            true
        }

        /// Decrease balance from the address.
        ///
        /// # Panics
        ///
        /// If `from` does not have enough balance.
        #[allow(unused)]
        fn burn_for(&mut self, from: Address, value: Balance) {
            let new_balance = self.balance_of_or_zero(&from) - value;
            self.balances.insert(from, new_balance);
            self.total_supply -= value;
            Self::emit_transfer(from, None, value);
        }

        /// Increase balance for the receiver out of nowhere.
        fn mint_for(&mut self, receiver: Address, value: Balance) {
            env::println(&format!(
                "Erc20::mint_for(receiver = {:?}, value = {:?})",
                receiver, value
            ));
            let new_balance = self.balance_of_or_zero(&receiver) + value;
            self.balances.insert(receiver, new_balance);
            self.total_supply += new_balance;
            Self::emit_transfer(None, receiver, new_balance);
        }
    }

    impl Erc20 {
        /// Emits an approval event.
        fn emit_approval(
            from: Address,
            to: Address,
            value: Balance,
        ) {
            assert_ne!(from, to);
            assert!(value > 0);
            deposit_event(Event::Approval { from, to, value });
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
            let (from, to) = (from.into(), to.into());
            assert!(from.is_some() || to.is_some());
            assert_ne!(from, to);
            assert!(value > 0);
            deposit_event(Event::Transfer { from, to, value });
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn it_works() {
        // `alice` is always the caller in this example!
        let mut erc20 = Erc20::deploy_mock(1234);
        let alice = Address::try_from([0x0; 32]).unwrap();
        let bob = Address::try_from([0x1; 32]).unwrap();
        assert_eq!(erc20.total_supply(), 1234);
        assert_eq!(erc20.balance_of(alice), 1234);
        assert_eq!(erc20.transfer_from(alice, bob, 234), true);
        assert_eq!(erc20.balance_of(alice), 1000);
        assert_eq!(erc20.balance_of(bob), 234);
        // Not allowed, since alice is the caller
        // and she has no approval from bob.
        assert_eq!(erc20.transfer_from(bob, alice, 1), false);
        assert_eq!(erc20.allowance(alice, bob), 0);
        assert_eq!(erc20.approve(bob, 10), true);
        // Bob is now doing the next calls
        env::test::set_caller(bob);
        assert_eq!(erc20.transfer_from(alice, bob, 15), false);
        assert_eq!(erc20.transfer_from(alice, bob, 10), true);
    }
}
