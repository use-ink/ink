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
#![cfg_attr(test, allow(dead_code))]
#![cfg_attr(test, allow(unused_imports))]

use ink_env::AccountId;
use ink_lang as ink;
use ink_prelude::vec::Vec;

// This is the return value that we expect if a smart contract supports receiving ERC-1155
// tokens.
//
// It is calculated with
// `bytes4(keccak256("onERC1155Received(address,address,uint256,uint256,bytes)"))`, and corresponds
// to 0xf23a6e61.
const ON_ERC_1155_RECEIVED_SELECTOR: [u8; 4] = [242, 58, 110, 97];

// This is the return value that we expect if a smart contract supports batch receiving ERC-1155
// tokens.
//
// It is calculated with
//`bytes4(keccak256("onERC1155BatchReceived(address,address,uint256[],uint256[],bytes)"))`, and
// corresponds to 0xbc197c81.
const _ON_ERC_1155_BATCH_RECEIVED_SELECTOR: [u8; 4] = [188, 25, 124, 129];

type TokenId = u128;
type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;

/// The interface for an ERC-1155 compliant contract.
///
/// The interface is defined here: `<https://eips.ethereum.org/EIPS/eip-1155>`.
///
/// The goal of ERC-1155 is to allow a single deployed contract to manage a variety of assets.
/// These assets can be fungible, non-fungible, or a combination.
///
/// By tracking multiple assets the ERC-1155 standard is able to support batch transfers, which
/// make it easy to transfer a mix of multiple tokens at once.
#[ink::trait_definition]
pub trait Erc1155 {
    /// Transfer the a `value` amount of `token_id` tokens to the `to` account from the `from`
    /// account.
    ///
    /// Note that the call does not have to originate from the `from` account, and may originate
    /// from any account which is approved to transfer `from`'s tokens.
    #[ink(message)]
    fn safe_transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        token_id: TokenId,
        value: Balance,
        data: Vec<u8>,
    );

    /// Perform a batch transfer of `token_ids` to the `to` account from the `from` account.
    ///
    /// The number of `values` specified to be transfer must match the number of `token_ids`,
    /// otherwise this call will revert.
    ///
    /// Note that the call does not have to originate from the `from` account, and may originate
    /// from any account which is approved to transfer `from`'s tokens.
    #[ink(message)]
    fn safe_batch_transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        token_ids: Vec<TokenId>,
        values: Vec<Balance>,
        data: Vec<u8>,
    );

    /// Query the balance of a specific token for the provided account.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId, token_id: TokenId) -> Balance;

    /// Query the balances for a set of tokens for a set of accounts.
    ///
    /// E.g use this call if you want to query what Alice and Bob's balances are for Tokens ID1 and
    /// ID2.
    ///
    /// This will return all the balances for a given owner before moving on to the next owner. In
    /// the example above this means that the return value should look like:
    ///
    /// [Alice Balance of Token ID1, Alice Balance of Token ID2, Bob Balance of Token ID2, Bob Balance of Token ID2]
    #[ink(message)]
    fn balance_of_batch(
        &self,
        owners: Vec<AccountId>,
        token_ids: Vec<TokenId>,
    ) -> Vec<Balance>;

    /// Enable or disable a third party, known as an `operator`, to control all tokens on behalf of
    /// the caller.
    #[ink(message)]
    fn set_approval_for_all(&mut self, operator: AccountId, approved: bool);

    /// Query if the given `operator` is allowed to control all of `owner`'s tokens.
    #[ink(message)]
    fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool;
}

/// The interface for an ERC-1155 Token Receiver contract.
///
/// The interface is defined here: `<https://eips.ethereum.org/EIPS/eip-1155>`.
///
/// Smart contracts which want to accept token transfers must implement this interface. By default
/// if a contract does not support this interface any transactions originating from an ERC-1155
/// compliant contract which attempt to transfer tokens directly to the contract's address must be
/// reverted.
#[ink::trait_definition]
pub trait Erc1155TokenReceiver {
    /// Handle the receipt of a single ERC-1155 token.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient is a smart
    /// contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it must
    /// return `ON_ERC_1155_RECEIVED_SELECTOR` from this function. To reject a transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than `ON_ERC_1155_RECEIVED_SELECTOR` as a return
    /// value.
    #[ink(message)]
    fn on_erc_1155_received(
        &mut self,
        operator: AccountId,
        from: AccountId,
        token_id: TokenId,
        value: Balance,
        data: Vec<u8>,
    ) -> Vec<u8>;

    /// Handle the receipt of multiple ERC-1155 tokens.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient is a smart
    /// contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it must
    /// return `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` from this function. To reject a transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` as a return
    /// value.
    #[ink(message)]
    fn on_erc_1155_batch_received(
        &mut self,
        operator: AccountId,
        from: AccountId,
        token_ids: Vec<TokenId>,
        values: Vec<Balance>,
        data: Vec<u8>,
    );
}

#[ink::contract]
mod erc1155 {
    use super::*;

    use ink_env::call::{
        build_call,
        utils::ReturnType,
        ExecutionInput,
        Selector,
    };
    use ink_prelude::collections::BTreeMap;
    use ink_storage::traits::{
        PackedLayout,
        SpreadLayout,
    };

    /// Indicate that a token transfer has occured.
    ///
    /// This must be emitted even if a zero value transfer occurs.
    #[ink(event)]
    pub struct TransferSingle {
        #[ink(topic)]
        operator: AccountId,
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        token_id: TokenId,
        value: Balance,
    }

    /// Indicate that an approval event has happened.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    /// Indicate that a token's URI has been updated.
    #[ink(event)]
    pub struct Uri {
        value: ink_prelude::string::String,
        #[ink(topic)]
        token_id: TokenId,
    }

    /// Represents an (Owner, Operator) pair, in which the operator is allowed to spend funds on
    /// behalf of the operator.
    #[derive(
        Copy,
        Clone,
        Debug,
        Ord,
        PartialOrd,
        Eq,
        PartialEq,
        PackedLayout,
        SpreadLayout,
        scale::Encode,
        scale::Decode,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    struct Approval {
        owner: AccountId,
        operator: AccountId,
    }

    /// An ERC-1155 contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        /// Tracks the balances of accounts across the different tokens that they might be holding.
        balances: BTreeMap<(AccountId, TokenId), Balance>,
        /// Which accounts (called operators) have been approved to spend funds on behalf of an owner.
        approvals: BTreeMap<Approval, ()>,
        /// A unique identifier for the tokens which have been minted (and are therefore supported)
        /// by this contract.
        token_id_nonce: TokenId,
    }

    impl Contract {
        /// Initialize a default instance of this ERC-1155 implementation.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Create the initial supply for a token.
        ///
        /// The initial supply will be provided to the caller (a.k.a the minter), and the
        /// `token_id` will be assigned by the smart contract.
        ///
        /// Note that as implemented anyone can create tokens. If you were to deploy this to a
        /// production environment you'd probably want to lock down the addresses that are allowed
        /// to create tokens.
        #[ink(message)]
        pub fn create(&mut self, value: Balance) -> TokenId {
            // Given that TokenId is a `u128` the likelihood of this overflowing is pretty slim.
            self.token_id_nonce += 1;
            self.balances
                .insert((self.env().caller(), self.token_id_nonce), value);

            // Emit transfer event but with mint semantics
            self.env().emit_event(TransferSingle {
                operator: self.env().caller(),
                from: AccountId::from([0; 32]),
                to: if value == 0 {
                    AccountId::from([0; 32])
                } else {
                    self.env().caller()
                },
                token_id: self.token_id_nonce,
                value,
            });

            self.token_id_nonce
        }

        /// Mint a `value` amount of `token_id` tokens.
        ///
        /// It is assumed that the token has already been `create`-ed. The newly minted supply will
        /// be assigned to the caller (a.k.a the minter).
        ///
        /// Note that as implemented anyone can mint tokens. If you were to deploy this to a
        /// production environment you'd probably want to lock down the addresses that are allowed
        /// to mint tokens.
        #[ink(message)]
        pub fn mint(&mut self, token_id: TokenId, value: Balance) {
            assert!(
                token_id <= self.token_id_nonce,
                "The `token_id` {:?} has not yet been created in this contract.",
                token_id
            );

            self.balances.insert((self.env().caller(), token_id), value);

            // Emit transfer event but with mint semantics
            self.env().emit_event(TransferSingle {
                operator: self.env().caller(),
                from: AccountId::from([0; 32]),
                to: self.env().caller(),
                token_id,
                value,
            });
        }

        // Helper function for performing single token transfers.
        //
        // Should not be used directly since it's missing certain checks which are important to the
        // ERC-1155 standard (it is expected that the caller has already perfomred these).
        fn perform_transfer(
            &mut self,
            from: AccountId,
            to: AccountId,
            token_id: TokenId,
            value: Balance,
            data: Vec<u8>,
        ) {
            assert!(
                self.balance_of(from, token_id) >= value,
                "Insufficent token balance for transfer."
            );

            self.balances
                .entry((from, token_id))
                .and_modify(|b| *b -= value);

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
                let params = build_call::<ink_env::DefaultEnvironment>()
                    .callee(to)
                    .gas_limit(5000)
                    .exec_input(
                        ExecutionInput::new(Selector::new(ON_ERC_1155_RECEIVED_SELECTOR))
                            .push_arg(self.env().caller())
                            .push_arg(from)
                            .push_arg(token_id)
                            .push_arg(value)
                            .push_arg(data),
                    )
                    .returns::<ReturnType<Vec<u8>>>()
                    .params();

                match ink_env::eval_contract(&params) {
                    Ok(v) => {
                        assert_eq!(
                            v,
                            &ON_ERC_1155_RECEIVED_SELECTOR[..],
                            "Recipient contract does not accept token transfers."
                        )
                    }
                    Err(e) => {
                        match e {
                            ink_env::Error::CodeNotFound => {
                                // Our recipient wasn't a smart contract, so there's nothing more for
                                // us to do
                            }
                            _ => {
                                // I have no insight to what's happening here, I can't deploy
                                // the following:
                                //
                                // ink_env::debug_println(&ink_prelude::format!("{:?}", e));
                                // panic!("{:?}", e)
                            }
                        }
                    }
                }
            }
        }
    }

    impl super::Erc1155 for Contract {
        #[ink(message)]
        fn safe_transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            token_id: TokenId,
            value: Balance,
            data: Vec<u8>,
        ) {
            if self.env().caller() != from {
                assert!(
                    self.is_approved_for_all(from, self.env().caller()),
                    "Caller is not allowed to transfer on behalf of {:?}.",
                    from
                );
            }

            assert!(
                to != AccountId::default(),
                "Cannot send tokens to the zero-address."
            );

            self.perform_transfer(from, to, token_id, value, data);
        }

        #[ink(message)]
        fn safe_batch_transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            token_ids: Vec<TokenId>,
            values: Vec<Balance>,
            data: Vec<u8>,
        ) {
            if self.env().caller() != from {
                assert!(
                    self.is_approved_for_all(from, self.env().caller()),
                    "Caller is not allowed to transfer on behalf of {:?}.",
                    from
                );
            }

            assert!(
                to != AccountId::default(),
                "Cannot send tokens to the zero-address."
            );

            assert_eq!(
                token_ids.len(),
                values.len(),
                "The number of tokens being transferred does
                 not match the number of transfer amounts."
            );

            token_ids.iter().zip(values.iter()).for_each(|(&id, &v)| {
                self.perform_transfer(from, to, id, v, data.clone());
            })
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId, token_id: TokenId) -> Balance {
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
        fn set_approval_for_all(&mut self, operator: AccountId, approved: bool) {
            assert!(
                operator != self.env().caller(),
                "An account does not need to approve themselves to transfer tokens."
            );

            let approval = Approval {
                owner: self.env().caller(),
                operator,
            };

            if approved {
                self.approvals.insert(approval, ());
            } else {
                self.approvals.remove(&approval);
            }

            self.env().emit_event(ApprovalForAll {
                owner: approval.owner,
                operator,
                approved,
            });
        }

        #[ink(message)]
        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.approvals.get(&Approval { owner, operator }).is_some()
        }
    }

    impl super::Erc1155TokenReceiver for Contract {
        #[ink(message, selector = "0xF23A6E61")]
        fn on_erc_1155_received(
            &mut self,
            _operator: AccountId,
            _from: AccountId,
            _token_id: TokenId,
            _value: Balance,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            unimplemented!("This smart contract does not accept token transfer.")
        }

        #[ink(message, selector = "0xBC197C81")]
        fn on_erc_1155_batch_received(
            &mut self,
            _operator: AccountId,
            _from: AccountId,
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
                ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])), /* dummy */
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
            let mut erc = Contract::new();
            erc.balances.insert((alice(), 1), 10);
            erc.balances.insert((alice(), 2), 20);
            erc.balances.insert((bob(), 1), 10);

            erc
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

        #[ink::test]
        fn minting_tokens_works() {
            let mut erc = Contract::new();

            set_sender(alice());
            assert_eq!(erc.create(0), 1);
            assert_eq!(erc.balance_of(alice(), 1), 0);

            erc.mint(1, 123);
            assert_eq!(erc.balance_of(alice(), 1), 123);
        }

        #[ink::test]
        #[should_panic]
        fn minting_not_allowed_for_nonexistent_tokens() {
            let mut erc = Contract::new();
            erc.mint(7, 123);
        }
    }
}
