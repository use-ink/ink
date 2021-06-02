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
    fn safe_transfer_from(
        &mut self,
        from: ink_env::AccountId,
        to: ink_env::AccountId,
        token_id: TokenId,
        value: Balance,
        data: Vec<u8>,
    );

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
    fn on_erc_1155_received(
        &mut self,
        operator: ink_env::AccountId,
        from: ink_env::AccountId,
        token_id: TokenId,
        value: Balance,
        data: Vec<u8>,
    ) -> Vec<u8>;

    #[ink(message)]
    fn on_erc_1155_batch_received(&mut self);
}

#[ink::contract]
mod erc1155 {
    use super::*;

    use ink_prelude::collections::BTreeMap;

    #[ink(event)]
    pub struct TransferSingle {
        operator: AccountId,
        from: AccountId,
        to: AccountId,
        token_id: TokenId,
        value: Balance,
    }

    /// An ERC-1155 contract.
    #[ink(storage)]
    pub struct Contract {
        balances: BTreeMap<(AccountId, TokenId), Balance>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(balances: BTreeMap<(AccountId, TokenId), Balance>) -> Self {
            Self { balances }
        }
    }

    impl super::Erc1155 for Contract {
        /*
            @notice Transfers `_value` amount of an `_id` from the `_from` address to the `_to` address specified (with safety call).
            @dev Caller must be approved to manage the tokens being transferred out of the `_from`
                 account (see "Approval" section of the standard).
            MUST revert if `_to` is the zero address.
            MUST revert if balance of holder for token `_id` is lower than the `_value` sent.
            MUST revert on any other error.
            MUST emit the `TransferSingle` event to reflect the balance change (see "Safe Transfer Rules" section of the standard).
                After the above conditions are met, this function MUST check if `_to` is a smart contract
                (e.g. code size > 0). If so, it MUST call `onERC1155Received` on `_to` and act
                appropriately (see "Safe Transfer Rules" section of the standard).

            @param _from    Source address
            @param _to      Target address
            @param _id      ID of the token type
            @param _value   Transfer amount
            @param _data    Additional data with no specified format, MUST be sent unaltered in call to `onERC1155Received` on `_to`
        */
        #[ink(message)]
        fn safe_transfer_from(
            &mut self,
            from: ink_env::AccountId,
            to: ink_env::AccountId,
            token_id: TokenId,
            value: Balance,
            data: Vec<u8>,
        ) {
            // TODO: Need to make sure self.env().caller is "Approved" on behalf of `from`

            // Q: Would a call be reverted if I return an Error vs. just panicking?
            assert!(
                to != AccountId::default(),
                "Cannot send tokens to the zero-address."
            );

            assert!(
                self.balance_of(from, token_id) >= value,
                "Insufficent token balance for transfer."
            );

            self.balances
                .get_mut(&(from, token_id))
                .and_then(|b| Some(*b -= value));

            self.balances
                .get_mut(&(to, token_id))
                .and_then(|b| Some(*b += value));

            self.env().emit_event(TransferSingle {
                operator: self.env().caller(),
                from: from.clone(),
                to: to.clone(),
                token_id,
                value,
            });

            // We call this _after_ the balance has been updated and the event has been fired
            //
            // Check if `to` is a smart contract
            // use ink_env::call::{build_call, ExecutionInput, Selector};
            // let magic_value = if let Err(e) = build_call::<ink_env::DefaultEnvironment>()
            //     .callee(to)
            //     .gas_limit(5000)
            //     .transferred_value(10)
            //     .exec_input(ExecutionInput::new(Selector::new([0; 4])))
            //     .returns::<()>()
            //     .fire()
            // {
            //     match e {
            //         ink_env::Error::CodeNotFound => self.on_erc_1155_received(
            //             self.env().caller(),
            //             from,
            //             token_id,
            //             value,
            //             data,
            //         ),
            //         _ => todo!("tbh, not sure"),
            //     }
            // } else {
            //     ink_prelude::vec![]
            // };

            // if magic_value != ink_prelude::vec![0] {
            //     todo!()
            // }
        }

        #[ink(message)]
        fn safe_batch_transfer_from(&mut self) {
            todo!()
        }

        #[ink(message)]
        fn balance_of(&self, owner: ink_env::AccountId, token_id: TokenId) -> Balance {
            *self.balances.get(&(owner, token_id)).unwrap_or(&0)
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
        fn on_erc_1155_received(
            &mut self,
            operator: ink_env::AccountId,
            from: ink_env::AccountId,
            token_id: TokenId,
            value: Balance,
            data: Vec<u8>,
        ) -> Vec<u8> {
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

        fn default_accounts(
        ) -> ink_env::test::DefaultAccounts<ink_env::DefaultEnvironment> {
            ink_env::test::default_accounts::<ink_env::DefaultEnvironment>()
                .expect("off-chain environment should have been initialized already")
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn init_contract() -> Contract {
            let mut balances = BTreeMap::new();
            balances.insert((alice(), 1), 10);
            balances.insert((alice(), 2), 20);
            balances.insert((bob(), 1), 10);

            Contract::new(balances)
        }

        #[ink::test]
        fn can_get_correct_balance_of() {
            let erc = init_contract();

            assert_eq!(erc.balance_of(alice(), 1), 10);
            assert_eq!(erc.balance_of(alice(), 2), 20);
            assert_eq!(erc.balance_of(alice(), 3), 0);
            assert_eq!(erc.balance_of(bob(), 2), 0);
        }

        #[ink::test]
        fn can_get_correct_batch_balance_of() {
            let erc = init_contract();

            assert_eq!(
                erc.balance_of_batch(vec![alice()], vec![1, 2, 3]),
                vec![10, 20, 0]
            );
            assert_eq!(
                erc.balance_of_batch(vec![alice(), bob()], vec![1]),
                vec![10, 10]
            );

            assert_eq!(
                erc.balance_of_batch(
                    vec![alice(), bob(), default_accounts().charlie],
                    vec![1, 2]
                ),
                vec![10, 20, 10, 0, 0, 0]
            );
        }

        #[ink::test]
        fn can_send_tokens_between_accounts() {
            let mut erc = init_contract();

            erc.safe_transfer_from(alice(), bob(), 1, 5, vec![]);
            assert_eq!(erc.balance_of(alice(), 1), 5);
            assert_eq!(erc.balance_of(bob(), 1), 15);
        }

        #[ink::test]
        #[should_panic]
        fn sending_too_many_tokens_fails() {
            let mut erc = init_contract();
            erc.safe_transfer_from(alice(), bob(), 1, 99, vec![]);
        }

        #[ink::test]
        #[should_panic]
        fn sending_tokens_to_zero_address_fails() {
            let burn: AccountId = [0; 32].into();

            let mut erc = init_contract();
            erc.safe_transfer_from(alice(), burn, 1, 10, vec![]);
        }
    }
}
