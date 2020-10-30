// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

//! A smart contract which demonstrates behavior of the `self.env().transfer()` function.
//! It transfers some of it's balance to the caller.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod give_me {
    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct GiveMe {}

    /// The error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the transfer failed.
        TransferFailed,
        /// Insufficient funds to execute transfer.
        InsufficientFunds,
        /// Transfer failed because it would have brought the contract's
        /// balance below the subsistence threshold.
        /// This is necessary to keep enough funds in the contract to
        /// allow for a tombstone to be created.
        BelowSubsistenceThreshold,
    }

    impl GiveMe {
        /// Creates a new instance of this contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Transfers `value` amount of tokens to the caller.
        ///
        /// # Errors
        ///
        /// - Returns `Error::InsufficientFunds` in case the requested transfer of
        ///   `value` exceeds the contracts balance.
        /// - Returns `Error::BelowSubsistenceThreshold` in case the requested transfer
        ///   of `value` would have brought the contract's balance below the subsistence
        ///   threshold.
        /// - Returns `Error::TransferFailed` in case the transfer failed for another
        ///   reason.
        #[ink(message)]
        pub fn give_me(&mut self, value: Balance) -> Result<(), Error> {
            if value > self.env().balance() {
                return Err(Error::InsufficientFunds)
            }
            self.env()
                .transfer(self.env().caller(), value)
                .map_err(|err| {
                    match err {
                        ink_env::Error::BelowSubsistenceThreshold => {
                            Error::BelowSubsistenceThreshold
                        }
                        _ => Error::TransferFailed,
                    }
                })
        }
    }

    impl Default for GiveMe {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_env::{
            call,
            test,
        };
        use ink_lang as ink;

        #[ink::test]
        fn transfer_works() {
            // given
            let contract_balance = 100;
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            assert_eq!(give_me.give_me(80), Ok(()));

            // then
            assert_eq!(get_balance(accounts.eve), 80);
        }

        #[ink::test]
        fn transfer_fails_insufficient_funds() {
            // given
            let contract_balance = 100;
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            let ret = give_me.give_me(120);

            // then
            assert_eq!(ret, Err(Error::InsufficientFunds));
        }

        /// Creates a new instance of `GiveMe` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> GiveMe {
            let accounts = default_accounts();
            let contract_id = ink_env::test::get_current_contract_account_id::<
                ink_env::DefaultEnvironment,
            >()
            .expect("Cannot get contract id");
            set_sender(accounts.alice);
            set_balance(contract_id, initial_balance);
            GiveMe::new()
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>()
                .unwrap_or([0x0; 32].into());
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("Off-chain environment should have been initialized already")
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account_id, balance,
            )
            .expect("Cannot set account balance");
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot set account balance")
        }
    }
}
