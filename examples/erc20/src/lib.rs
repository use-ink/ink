// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract(version = "0.1.0")]
mod erc20 {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage2::{
        lazy::Lazy,
        collections::HashMap as StorageHashMap,
    };

    #[ink(storage)]
    struct Erc20 {
        total_supply: Lazy<Balance>,
        balances: StorageHashMap<AccountId, Balance>,
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    #[ink(event)]
    struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    impl Erc20 {
        #[ink(constructor)]
        fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);
            let instance = Self {
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: StorageHashMap::new(),
            };
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });
            instance
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            let from = self.env().caller();
            self.transfer_from_to(from, to, value)
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            true
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> bool {
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return false
            }
            self.allowances.insert((from, caller), allowance - value);
            self.transfer_from_to(from, to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> bool {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return false
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);
            self.env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
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

    /// Unit tests.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_core::env;

        /// Executes the given test through the off-chain environment.
        fn run_test<F>(test_fn: F)
        where
            F: FnOnce(),
        {
            env::test::run_test::<env::DefaultEnvTypes, _>(|_| {
                test_fn();
                Ok(())
            }).unwrap()
        }

        /// The default constructor does its job.
        #[test]
        fn new_works() {
            run_test(|| {
                // Constructor works.
                let _erc20 = Erc20::new(100);

                // Transfer event triggered during initial contruction.
                let emitted_events = env::test::recorded_events().collect::<Vec<_>>();
                assert_eq!(1, emitted_events.len());
                let raw_event = emitted_events.first().unwrap();
                let event = <Event as scale::Decode>::decode(&mut &raw_event.data[..])
                    .expect("Invalid contract Event");
                if let Event::Transfer(transfer) = event {
                    assert_eq!(100, transfer.value);
                } else {
                    panic!("Expected a Transfer Event")
                }
            })
        }

        /// The total supply was applied.
        #[test]
        fn total_supply_works() {
            run_test(|| {
                // Constructor works.
                let erc20 = Erc20::new(100);
                // Transfer event triggered during initial contruction.
                assert_eq!(env::test::recorded_events().count(), 1);
                // Get the token total supply.
                assert_eq!(erc20.total_supply(), 100);
            })
        }

        /// Get the actual balance of an account.
        #[test]
        fn balance_of_works() {
            run_test(|| {
                // Constructor works
                let erc20 = Erc20::new(100);
                // Transfer event triggered during initial contruction
                assert_eq!(env::test::recorded_events().count(), 1);
                let accounts = env::test::default_accounts::<env::DefaultEnvTypes>()
                    .expect("Cannot get accounts");
                // Alice owns all the tokens on deployment
                assert_eq!(erc20.balance_of(accounts.alice), 100);
                // Bob does not owns tokens
                assert_eq!(erc20.balance_of(accounts.bob), 0);
            })
        }

        #[test]
        fn transfer_works() {
            run_test(|| {
                // Constructor works.
                let mut erc20 = Erc20::new(100);
                // Transfer event triggered during initial contruction.
                assert_eq!(1, env::test::recorded_events().count());
                let accounts = env::test::default_accounts::<env::DefaultEnvTypes>()
                    .expect("Cannot get accounts");

                assert_eq!(erc20.balance_of(accounts.bob), 0);
                // Alice transfers 10 tokens to Bob.
                assert_eq!(erc20.transfer(accounts.bob, 10), true);
                // The second Transfer event takes place.
                assert_eq!(2, env::test::recorded_events().count());
                // Bob owns 10 tokens.
                assert_eq!(erc20.balance_of(accounts.bob), 10);
            })
        }

        #[test]
        fn invalid_transfer_should_fail() {
            run_test(|| {
                // Constructor works.
                let mut erc20 = Erc20::new(100);
                // Transfer event triggered during initial contruction.
                assert_eq!(env::test::recorded_events().count(), 1);
                let accounts = env::test::default_accounts::<env::DefaultEnvTypes>()
                    .expect("Cannot get accounts");

                assert_eq!(erc20.balance_of(accounts.bob), 0);
                // Get contract address.
                let callee =
                    env::account_id::<env::DefaultEnvTypes>().unwrap_or([0x0; 32].into());
                // Create call
                let mut data =
                    env::call::CallData::new(env::call::Selector::from_str("balance_of"));
                data.push_arg(&accounts.bob);
                // Push the new execution context to set Bob as caller
                assert_eq!(
                    env::test::push_execution_context::<env::DefaultEnvTypes>(
                        accounts.bob,
                        callee,
                        1000000,
                        1000000,
                        data
                    ),
                    ()
                );

                // Bob fails to transfers 10 tokens to Eve.
                assert_eq!(erc20.transfer(accounts.eve, 10), false);
                // Alice owns all the tokens.
                assert_eq!(erc20.balance_of(accounts.alice), 100);
                assert_eq!(erc20.balance_of(accounts.bob), 0);
                assert_eq!(erc20.balance_of(accounts.eve), 0);
            })
        }

        #[test]
        fn transfer_from_works() {
            run_test(|| {
                // Constructor works.
                let mut erc20 = Erc20::new(100);
                // Transfer event triggered during initial contruction.
                assert_eq!(env::test::recorded_events().count(), 1);
                let accounts = env::test::default_accounts::<env::DefaultEnvTypes>()
                    .expect("Cannot get accounts");

                // Bob fails to transfer tokens owned by Alice.
                assert_eq!(erc20.transfer_from(accounts.alice, accounts.eve, 10), false);
                // Alice approves Bob for token transfers on her behalf.
                assert_eq!(erc20.approve(accounts.bob, 10), true);

                // The approve event takes place.
                assert_eq!(env::test::recorded_events().count(), 2);

                // Get contract address.
                let callee =
                    env::account_id::<env::DefaultEnvTypes>().unwrap_or([0x0; 32].into());
                // Create call.
                let mut data =
                    env::call::CallData::new(env::call::Selector::from_str("balance_of"));
                data.push_arg(&accounts.bob);
                // Push the new execution context to set Bob as caller.
                assert_eq!(
                    env::test::push_execution_context::<env::DefaultEnvTypes>(
                        accounts.bob,
                        callee,
                        1000000,
                        1000000,
                        data
                    ),
                    ()
                );

                // Bob transfers tokens from Alice to Eve.
                assert_eq!(erc20.transfer_from(accounts.alice, accounts.eve, 10), true);
                // The third event takes place.
                assert_eq!(env::test::recorded_events().count(), 3);
                // Eve owns tokens.
                assert_eq!(erc20.balance_of(accounts.eve), 10);
            })
        }
    }
}
