// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::contract]
pub mod give_me {
    /// No storage is needed for this simple contract.
    #[ink(storage)]
    pub struct GiveMe {}

    /// The error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
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
        /// - Panics in case the requested transfer exceeds the contract balance.
        /// - Panics in case the requested transfer would have brought the
        ///   contract balance below the subsistence threshold.
        /// - Panics in case the transfer failed for another reason.
        #[ink(message)]
        pub fn give_me(&mut self, value: Balance) {
            ink_env::debug_println!("requested value: {}", value);
            ink_env::debug_println!("contract balance: {}", self.env().balance());

            assert!(value <= self.env().balance(), "insufficient funds!");

            match self.env().transfer(self.env().caller(), value) {
                Err(ink_env::Error::BelowSubsistenceThreshold) => {
                    panic!(
                        "requested transfer would have brought contract\
                        below subsistence threshold!"
                    )
                }
                Err(_) => panic!("transfer failed!"),
                Ok(_) => {}
            }
        }

        /// Asserts that the token amount sent as payment with this call
        /// is exactly `10`. This method will fail otherwise, and the
        /// transaction would then be reverted.
        ///
        /// # Note
        ///
        /// The method needs to be annotated with `payable`; only then it is
        /// allowed to receive value as part of the call.
        #[ink(message, payable, selector = "0xCAFEBABE")]
        pub fn was_it_ten(&self) {
            ink_env::debug_println!(
                "received payment: {}",
                self.env().transferred_balance()
            );
            assert!(
                self.env().transferred_balance() == 10,
                "payment was not ten"
            );
        }
    }

    #[cfg(not(feature = "ink-experimental-engine"))]
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
            give_me.give_me(80);

            // then
            assert_eq!(get_balance(accounts.eve), 80);
        }

        #[ink::test]
        #[should_panic(expected = "insufficient funds!")]
        fn transfer_fails_insufficient_funds() {
            // given
            let contract_balance = 100;
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            give_me.give_me(120);

            // then
            // `give_me` must already have panicked here
        }

        #[ink::test]
        fn test_transferred_value() {
            // given
            let accounts = default_accounts();
            let give_me = create_contract(100);

            // when
            set_sender(accounts.eve);
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.eve);
            let mock_transferred_balance = 10;

            // Push the new execution context which sets Eve as caller and
            // the `mock_transferred_balance` as the value which the contract
            // will see as transferred to it.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.eve,
                contract_id(),
                1000000,
                mock_transferred_balance,
                data,
            );

            // then
            // there must be no panic
            give_me.was_it_ten();
        }

        #[ink::test]
        #[should_panic(expected = "payment was not ten")]
        fn test_transferred_value_must_fail() {
            // given
            let accounts = default_accounts();
            let give_me = create_contract(100);

            // when
            set_sender(accounts.eve);
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([
                0xCA, 0xFE, 0xBA, 0xBE,
            ]));
            data.push_arg(&accounts.eve);
            let mock_transferred_balance = 13;

            // Push the new execution context which sets Eve as caller and
            // the `mock_transferred_balance` as the value which the contract
            // will see as transferred to it.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.eve,
                contract_id(),
                1000000,
                mock_transferred_balance,
                data,
            );

            // then
            give_me.was_it_ten();
        }

        /// Creates a new instance of `GiveMe` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> GiveMe {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            GiveMe::new()
        }

        fn contract_id() -> AccountId {
            ink_env::test::get_current_contract_account_id::<ink_env::DefaultEnvironment>(
            )
            .expect("Cannot get contract id")
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

    #[cfg(feature = "ink-experimental-engine")]
    #[cfg(test)]
    mod tests_experimental_engine {
        use super::*;
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
        #[should_panic(expected = "insufficient funds!")]
        fn transfer_fails_insufficient_funds() {
            // given
            let contract_balance = 100;
            let accounts = default_accounts();
            let mut give_me = create_contract(contract_balance);

            // when
            set_sender(accounts.eve);
            give_me.give_me(120);

            // then
            // `give_me` must already have panicked here
        }

        #[ink::test]
        fn test_transferred_value() {
            // given
            let accounts = default_accounts();
            let give_me = create_contract(100);

            // when
            // Push the new execution context which sets Eve as caller and
            // the `mock_transferred_balance` as the value which the contract
            // will see as transferred to it.
            set_sender(accounts.eve);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(10);

            // then
            // there must be no panic
            give_me.was_it_ten();
        }

        #[ink::test]
        #[should_panic(expected = "payment was not ten")]
        fn test_transferred_value_must_fail() {
            // given
            let accounts = default_accounts();
            let give_me = create_contract(100);

            // when
            // Push the new execution context which sets Eve as caller and
            // the `mock_transferred_balance` as the value which the contract
            // will see as transferred to it.
            set_sender(accounts.eve);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(13);

            // then
            give_me.was_it_ten();
        }

        /// Creates a new instance of `GiveMe` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> GiveMe {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            GiveMe::new()
        }

        fn contract_id() -> AccountId {
            ink_env::test::callee::<ink_env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(sender);
        }

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink_env::test::set_account_balance::<ink_env::DefaultEnvironment>(
                account_id, balance,
            )
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink_env::test::get_account_balance::<ink_env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }
    }
}
