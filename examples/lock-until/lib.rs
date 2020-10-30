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

//! Locks the tokens sent to this smart contract until a lock period
//! is reached. The lock period is set by the initial creator of the
//! contract.
//!
//! Whoever calls `unlock()` first gets the locked amount paid out.
//! The contract terminates itself subsequently.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod lock_until {
    /// Hold a timestamp denoting how long the balance in this
    /// contract should be kept locked.
    #[ink(storage)]
    pub struct LockUntil {
        lock_until: Timestamp,
    }

    /// The error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if the lock period is not yet over.
        LockPeriodNotOver,
    }

    impl LockUntil {
        /// Creates a new lock-until smart contract. The tokens sent to this contract
        /// will be locked until the supplied `lock_until` timestamp has been reached.
        #[ink(constructor)]
        pub fn new(lock_until: Timestamp) -> Self {
            Self { lock_until }
        }

        /// Locks the amount sent with this call.
        ///
        /// If the contract already has some balance then the value send with
        /// this call is just added to the existing balance.
        #[ink(message, payable)]
        pub fn lock(&mut self) {}

        /// Tries to unlock the currently locked value.
        ///
        /// # Errors
        ///
        /// - Return `Error::LockPeriodNotOver` in case the timestamp of this
        ///   block is >= to the timestamp set when creating this contract.
        #[ink(message)]
        pub fn unlock(&mut self) -> Result<(), Error> {
            let now = self.env().block_timestamp();
            if now < self.lock_until {
                return Err(Error::LockPeriodNotOver)
            }
            self.env().terminate_contract(self.env().caller());
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
        fn unlocking_before_end_does_not_work() {
            // given
            let mut lock_until = create_contract(100);

            // when
            let maybe_unlocked = lock_until.unlock();

            // then
            // the amount has been locked until `now() + 1`, but since the
            // block has not been advanced `now()` still returns the same
            // block timestamp.
            assert_eq!(maybe_unlocked, Err(Error::LockPeriodNotOver));
        }

        #[ink::test]
        fn unlocking_works() {
            // given
            let contract_balance = 100;
            let accounts = default_accounts();
            let mut lock_until = create_contract(contract_balance);
            assert_eq!(lock_until.unlock(), Err(Error::LockPeriodNotOver));
            set_sender(accounts.eve);

            // when
            // in order to change the result of `now()` the block needs to
            // be advanced.
            ink_env::test::advance_block::<ink_env::DefaultEnvironment>()
                .expect("Cannot advance block");

            // then
            let should_terminate = move || lock_until.unlock();
            ink_env::assert_contract_termination!(should_terminate, accounts.eve, 100);
        }

        /// Creates a new instance of `LockUntil` with `initial_balance`.
        /// The created contract is set to lock until `now() + 1`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> LockUntil {
            let accounts = default_accounts();
            let contract_id = ink_env::test::get_current_contract_account_id::<
                ink_env::DefaultEnvironment,
            >()
            .expect("Cannot get contract id");

            set_sender(accounts.alice);
            set_balance(contract_id, initial_balance);
            set_balance(accounts.eve, 200);

            LockUntil::new(now() + 1)
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

        fn now() -> Timestamp {
            ink_env::block_timestamp::<ink_env::DefaultEnvironment>()
                .expect("Cannot get block timestamp")
        }
    }
}
