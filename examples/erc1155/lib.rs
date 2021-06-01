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

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;
use ink_prelude::vec::Vec;

type TokenId = u128;
type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;

#[ink::trait_definition]
pub trait Erc1155 {
    #[ink(message)]
    fn safe_transfer_from(&mut self);

    #[ink(message)]
    fn safe_batch_transfer_from(&mut self);

    #[ink(message)]
    fn balance_of(&self, owner: ink_env::AccountId, token_id: TokenId) -> Balance;

    #[ink(message)]
    fn balance_of_batch(
        &self,
        owners: Vec<ink_env::AccountId>,
        token_ids: Vec<TokenId>,
    ) -> Vec<Balance>;

    #[ink(message)]
    fn set_approval_for_all(&mut self);

    #[ink(message)]
    fn is_approved_for_all(&self);
}

#[ink::trait_definition]
pub trait Erc1155TokenReceiver {
    #[ink(message)]
    fn on_erc_1155_received(&mut self);

    #[ink(message)]
    fn on_erc_1155_batch_received(&mut self);
}

#[ink::contract]
mod erc1155 {
    use super::*;

    use ink_prelude::collections::BTreeMap;

    /// An ERC-1155 contract.
    #[ink(storage)]
    pub struct Contract {
        balances: BTreeMap<AccountId, BTreeMap<TokenId, Balance>>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(balances: BTreeMap<AccountId, BTreeMap<TokenId, Balance>>) -> Self {
            Self { balances }
        }
    }

    impl super::Erc1155 for Contract {
        #[ink(message)]
        fn safe_transfer_from(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn safe_batch_transfer_from(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn balance_of(&self, owner: ink_env::AccountId, token_id: TokenId) -> Balance {
            *self
                .balances
                .get(&owner)
                .and_then(|b| b.get(&token_id))
                .unwrap_or(&0)
        }

        #[ink(message)]
        fn balance_of_batch(
            &self,
            owners: Vec<AccountId>,
            token_ids: Vec<TokenId>,
        ) -> Vec<Balance> {
            let mut output = Vec::new();
            for o in &owners {
                for t in &token_ids {
                    let amt = self.balance_of(*o, *t);
                    output.push(amt);
                }
            }
            output
        }

        #[ink(message)]
        fn set_approval_for_all(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn is_approved_for_all(&self) {
            todo!()
        }
    }

    impl super::Erc1155TokenReceiver for Contract {
        #[ink(message)]
        fn on_erc_1155_received(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn on_erc_1155_batch_received(&mut self) {
            todo!()
        }
    }

    /// Unit tests.
    #[cfg(not(feature = "ink-experimental-engine"))]
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use crate::Erc1155;

        use ink_lang as ink;

        #[ink::test]
        fn can_set_and_get_balances() {
            let alice: AccountId = [1; 32].into();
            let bob: AccountId = [2; 32].into();

            let mut balances = BTreeMap::new();
            balances.insert(1, 10);
            balances.insert(2, 20);

            let mut accounts = BTreeMap::new();
            accounts.insert(alice, balances);

            let erc = Contract::new(accounts);

            assert_eq!(erc.balance_of(alice, 1), 10);
            assert_eq!(erc.balance_of(alice, 2), 20);
            assert_eq!(erc.balance_of(alice, 3), 0);
            assert_eq!(erc.balance_of(bob, 1), 0);
        }

        #[ink::test]
        fn can_set_and_get_batch_balances() {
            let alice: AccountId = [1; 32].into();
            let bob: AccountId = [2; 32].into();
            let charlie: AccountId = [3; 32].into();

            let mut balances = BTreeMap::new();
            balances.insert(1, 10);
            balances.insert(2, 20);

            let mut accounts = BTreeMap::new();
            accounts.insert(alice, balances.clone());
            accounts.insert(bob, balances);

            let erc = Contract::new(accounts);

            assert_eq!(
                erc.balance_of_batch(vec![alice], vec![1, 2, 3]),
                vec![10, 20, 0]
            );
            assert_eq!(
                erc.balance_of_batch(vec![alice, bob], vec![1]),
                vec![10, 10]
            );

            assert_eq!(
                erc.balance_of_batch(vec![alice, bob, charlie], vec![1, 2]),
                vec![10, 20, 10, 20, 0, 0]
            );
        }
    }
}
