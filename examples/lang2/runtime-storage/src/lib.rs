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

use scale::Encode as _;
use ink_core::{
    memory::format,
    storage,
};
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod runtime {
    /// This simple contract reads a value from runtime storage
    #[ink(storage)]
    struct RuntimeStorage {
        balance_storage_keys: storage::HashMap<AccountId, [u8; 32]>,
    }

    impl RuntimeStorage {
        #[ink(constructor)]
        fn new(&mut self) {}

        #[ink(message)]
        fn add_account_storage_key(&mut self, account: AccountId, key: [u8; 32]) {
            self.env().println(&format!("Adding key for account {:?}", account.encode()));
            self.balance_storage_keys.insert(account, key);
        }

        /// Returns the account balance, read directly from runtime storage
        #[ink(message)]
        fn get_balance(&self, account: AccountId) -> Balance {
            self.env().println(&format!("Getting balance for account {:?}", account.encode()));
            match self.balance_storage_keys.get(&account) {
                Some(key) => {
                    let result = self.env().get_runtime_storage::<Balance>(&key[..]);
                    match result {
                        Ok(balance) => {
                            self.env().println("get_runtime_storage: Read balance Ok");
                            balance
                        },
                        Err(err) => {
                            self.env().println(&format!("Error reading runtime storage {:?}", err));
                            0
                        },
                    }
                }
                None => {
                    self.env().println(&format!("No key found for account {:?}", account.encode()));
                    0
                }
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
