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
        fn new(&mut self) {}

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            self.env().println("get_balance");
            const BALANCE_OF: &[u8] = b"balance:";
            let key = account.to_keyed_vec(BALANCE_OF);
            self.env().println("constructed key");
            match self.env().get_runtime_storage::<Balance>(&key) {
                Ok(balance) => {
                    self.env().println("get_runtime_storage: Read balance Ok");
                    balance
                },
                Err(_) => {
                    self.env().println("get_runtime_storage: Error reading balance");
                    0
                },
            }
        }
    }

//    #[cfg(all(test))]
//    mod tests {
//        use super::*;
//
//        #[test]
//        fn it_works() {
//            let contract = RuntimeStorage::new();
//            assert_eq!(contract.get_balance(), false);
//        }
//    }
}
