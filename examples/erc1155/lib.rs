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

// This is the "magic" return value that we expect if a smart contract supports receiving ERC-1155
// tokens.
//
// It is calculated with
// `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))`, and corresponds
// to 0xf23a6e61.
//
// Note that this is Ethereum specific, I don't know how it translates in Ink! land.
const MAGIC_VALUE: [u8; 4] = [242, 58, 110, 97];

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
    fn safe_batch_transfer_from(
        &mut self,
        from: ink_env::AccountId,
        to: ink_env::AccountId,
        token_ids: Vec<TokenId>,
        values: Vec<Balance>,
        data: Vec<u8>,
    );

    #[ink(message)]
    fn balance_of(&self, owner: ink_env::AccountId, token_id: TokenId) -> Balance;

    #[ink(message)]
    fn balance_of_batch(
        &self,
        owners: Vec<ink_env::AccountId>,
        token_ids: Vec<TokenId>,
    ) -> Vec<Balance>;

    #[ink(message)]
    fn set_approval_for_all(&mut self, operator: ink_env::AccountId, approved: bool);

    #[ink(message)]
    fn is_approved_for_all(
        &self,
        owner: ink_env::AccountId,
        operator: ink_env::AccountId,
    ) -> bool;
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
    fn on_erc_1155_batch_received(
        &mut self,
        operator: ink_env::AccountId,
        from: ink_env::AccountId,
        token_ids: Vec<TokenId>,
        values: Vec<Balance>,
        data: Vec<u8>,
    );
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

    #[ink(event)]
    pub struct ApprovalForAll {
        owner: AccountId,
        operator: AccountId,
        approved: bool,
    }

    /// An ERC-1155 contract.
    #[ink(storage)]
    pub struct Contract {
        balances: BTreeMap<(AccountId, TokenId), Balance>,

        /// Which accounts (called operators) have been approved to spend funds on behalf of an owner.
        ///
        /// Note that the mapping is Set<(Owner, Operator)>.
        ///
        /// TODO: Figure out why I can't use a Set here...
        approvals: BTreeMap<(AccountId, AccountId), ()>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new(balances: BTreeMap<(AccountId, TokenId), Balance>) -> Self {
            Self {
                balances,
                approvals: Default::default(),
            }
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
            // Q: Does the caller change if the function is called from within this smart contract?
            if self.env().caller() != from {
                assert!(
                    self.is_approved_for_all(from, self.env().caller()),
                    "Caller is not allowed to transfer on behalf of {:?}.",
                    from
                );
            }

            // Q: Would a call be reverted if I return an Error vs. just panicking?
            assert!(
                to != AccountId::default(),
                "Cannot send tokens to the zero-address."
            );

            assert!(
                self.balance_of(from, token_id) >= value,
                "Insufficent token balance for transfer."
            );

            if let Some(b) = self.balances.get_mut(&(from, token_id)) {
                *b -= value
            }

            self.balances
                .entry((to, token_id))
                .and_modify(|b| *b += value)
                .or_insert(value);

            self.env().emit_event(TransferSingle {
                operator: self.env().caller(),
                from,
                to,
                token_id,
                value,
            });

            // Quick Haxx, otherwise my tests just panic due to the use of eval_contract()
            #[cfg(not(test))]
            {
                // If our recipient is a smart contract we need to see if they accept or
                // reject this transfer. If they reject it we need to revert the call.
                let params = ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                    .callee(to)
                    .gas_limit(5000) // what's the correct amount to use here?
                    .exec_input(
                        // Idk how to get the bytes for the selector
                        ink_env::call::ExecutionInput::new(ink_env::call::Selector::new([166, 229, 27, 154]))
                        .push_arg(self.env().caller())
                        .push_arg(from)
                        .push_arg(token_id)
                        .push_arg(value)
                        .push_arg(data)
                    )
                    .returns::<ink_env::call::utils::ReturnType<Vec<u8>>>()
                    .params();

                match ink_env::eval_contract(&params) {
                    Ok(v) => assert_eq!(
                        v,
                        &MAGIC_VALUE[..],
                        "Recipient contract does not accept token transfers."
                    ),
                    Err(e) => match e {
                        ink_env::Error::CodeNotFound => {
                            // Our recipient wasn't a smart contract, so there's nothing more for
                            // us to do
                        }
                        _ => panic!("{:?}", e),
                    },
                }
            }
        }

        #[ink(message)]
        fn safe_batch_transfer_from(
            &mut self,
            from: ink_env::AccountId,
            to: ink_env::AccountId,
            token_ids: Vec<TokenId>,
            values: Vec<Balance>,
            data: Vec<u8>,
        ) {
            assert_eq!(
                token_ids.len(),
                values.len(),
                "The number of tokens being transferred does
                 not match the number of transfer amounts."
            );

            token_ids.iter().zip(values.iter()).for_each(|(&id, &v)| {
                self.safe_transfer_from(from, to, id, v, data.clone());
            })
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
        fn set_approval_for_all(&mut self, operator: ink_env::AccountId, approved: bool) {
            if approved {
                self.approvals.insert((self.env().caller(), operator), ());
            } else {
                self.approvals.remove(&(self.env().caller(), operator));
            }

            self.env().emit_event(ApprovalForAll {
                owner: self.env().caller(),
                operator,
                approved,
            });
        }

        #[ink(message)]
        fn is_approved_for_all(
            &self,
            owner: ink_env::AccountId,
            operator: ink_env::AccountId,
        ) -> bool {
            self.approvals.get(&(owner, operator)).is_some()
        }
    }

    impl super::Erc1155TokenReceiver for Contract {
        #[ink(message)]
        fn on_erc_1155_received(
            &mut self,
            _operator: ink_env::AccountId,
            _from: ink_env::AccountId,
            _token_id: TokenId,
            _value: Balance,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            unimplemented!("This smart contract does not accept token transfer.")
        }

        #[ink(message)]
        fn on_erc_1155_batch_received(
            &mut self,
            _operator: ink_env::AccountId,
            _from: ink_env::AccountId,
            _token_ids: Vec<TokenId>,
            _values: Vec<Balance>,
            _data: Vec<u8>,
        ) {
            unimplemented!("This smart contract does not accept batch token transfers.")
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

        fn set_sender(sender: AccountId) {
            const WALLET: [u8; 32] = [7; 32];
            ink_env::test::push_execution_context::<Environment>(
                sender,
                WALLET.into(),
                1000000,
                1000000,
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])), // dummy
            );
        }

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

        fn charlie() -> AccountId {
            default_accounts().charlie
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
                erc.balance_of_batch(vec![alice(), bob(), charlie()], vec![1, 2]),
                vec![10, 20, 10, 0, 0, 0]
            );
        }

        #[ink::test]
        fn can_send_tokens_between_accounts() {
            let mut erc = init_contract();

            erc.safe_transfer_from(alice(), bob(), 1, 5, vec![]);
            assert_eq!(erc.balance_of(alice(), 1), 5);
            assert_eq!(erc.balance_of(bob(), 1), 15);

            erc.safe_transfer_from(alice(), bob(), 2, 5, vec![]);
            assert_eq!(erc.balance_of(alice(), 2), 15);
            assert_eq!(erc.balance_of(bob(), 2), 5);
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

        #[ink::test]
        fn can_send_batch_tokens() {
            let mut erc = init_contract();
            erc.safe_batch_transfer_from(alice(), bob(), vec![1, 2], vec![5, 10], vec![]);

            let balances = erc.balance_of_batch(vec![alice(), bob()], vec![1, 2]);
            assert_eq!(balances, vec![5, 10, 15, 10])
        }

        #[ink::test]
        #[should_panic]
        fn rejects_batch_if_lengths_dont_match() {
            let mut erc = init_contract();
            erc.safe_batch_transfer_from(alice(), bob(), vec![1, 2, 3], vec![5], vec![]);
        }

        #[ink::test]
        fn operator_can_send_tokens() {
            let mut erc = init_contract();

            let owner = alice();
            let operator = bob();

            set_sender(owner);
            erc.set_approval_for_all(operator, true);

            set_sender(operator);
            erc.safe_transfer_from(owner, charlie(), 1, 5, vec![]);
            assert_eq!(erc.balance_of(alice(), 1), 5);
            assert_eq!(erc.balance_of(charlie(), 1), 5);
        }

        #[ink::test]
        fn approvals_work() {
            let mut erc = init_contract();
            let owner = alice();
            let operator = bob();
            let another_operator = charlie();

            // Note: All of these tests are from the context of the owner who is either allowing or
            // disallowing an operator to control their funds.
            set_sender(owner);
            assert!(erc.is_approved_for_all(owner, operator) == false);

            erc.set_approval_for_all(operator, true);
            assert!(erc.is_approved_for_all(owner, operator));

            erc.set_approval_for_all(another_operator, true);
            assert!(erc.is_approved_for_all(owner, another_operator));

            erc.set_approval_for_all(operator, false);
            assert!(erc.is_approved_for_all(owner, operator) == false);
        }
    }
}
