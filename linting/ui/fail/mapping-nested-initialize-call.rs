// Copyright 2018-2022 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod my_contract {
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct MyContract {
        balances: Mapping<AccountId, Balance>,
    }

    impl MyContract {
        /// The linter currently does not detect if `initialize_contract` is
        /// called in a nested function.
        #[ink(constructor)]
        pub fn new1() -> Self {
            Self::foo()
        }

        fn foo() -> Self {
            ink_lang::utils::initialize_contract(Self::new_init)
        }

        /// Default initializes the contract.
        fn new_init(&mut self) {
            let caller = Self::env().caller();
            let value: Balance = Default::default();
            self.balances.insert(&caller, &value);
        }

        /// Returns something.
        #[ink(message)]
        pub fn get(&self) {
            // ...
        }
    }
}

fn main() {}
