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

#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::storage;
use ink_lang2 as ink;
use scale::KeyedVec as _;

#[ink::contract(version = "0.1.0")]
mod runtime {
    /// This simple contract reads a value from runtime storage
    #[ink(storage)]
    struct RuntimeStorage {
    }

    impl RuntimeStorage {
        #[ink(constructor)]
        fn default(&mut self) {
        }

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            const BALANCE_OF: &[u8] = b"balance:";
            let key = account.to_keyed_vec(BALANCE_OF);
            match env.runtime_get_storage::<Balance>(&key) {
                Some(Ok(balance)) => balance,
                Some(Err(_)) => {
                    env.println("Error decoding balance");
                    0
                },
                None => {
                    env.println("Balance for account not found");
                    0
                }
            }
        }
    }

    #[cfg(all(test))]
    mod tests {
        use super::*;

        #[test]
        fn it_works() {
//            let contract = RuntimeStorage::default();
//            assert_eq!(contract.get_balance(), false);
        }
    }
}
