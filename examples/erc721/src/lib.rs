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
mod erc721 {
    use ink_core::storage;
    use scale::{Decode, Encode};

    /// A token ID.
    pub type TokenId = u32;

    #[ink(storage)]
    struct Erc721 {
        /// Mapping from token to owner.
        token_owner: storage::HashMap<TokenId, AccountId>,
        /// Mapping from token to approvals users.
        token_approvals: storage::HashMap<TokenId, AccountId>,
        /// Mapping from owner to number of owned token.
        owned_tokens_count: storage::HashMap<AccountId, u32>,
        /// Mapping from owner to operator approvals
        operator_approvals: storage::HashMap<(AccountId, AccountId), bool>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
    pub enum Error {
        NotOwner,
        NotApproved,
        TokenExists,
        TokenNotFound,
        CanNotInsert,
        CanNotRemove,
        CanNotGetCounter,
        ZeroAccountNotAllowed,
    }

    /// Event emitted when a token transfer occurs
    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emited when a token approve occurs
    #[ink(event)]
    struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    impl Erc721 {
        #[ink(constructor)]
        fn new(&mut self) {}

        /// Get token balance of specific account.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            self.balance_of_or_zero(&owner)
        }

        /// Get token owner.
        #[ink(message)]
        fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&id).cloned()
        }

        /// The approved address for this token, or the none address if there is none
        #[ink(message)]
        fn approved_for(&self, id: TokenId) -> Option<AccountId> {
            self.token_approvals.get(&id).cloned()
        }

        /// Mints a new token.
        #[ink(message)]
        fn mint(&mut self, id: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self.add_token_to(&caller, &id)?;
            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(caller),
                id,
            });
            Ok(())
        }

        /// Burns an existing token. Only the owner can burn the token.
        #[ink(message)]
        fn burn(&mut self, id: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.token_owner.get(&id) != Some(&caller) {
                return Err(Error::NotOwner);
            };
            self.remove_token_from(&caller, &id)?;
            self.env().emit_event(Transfer {
                from: Some(caller),
                to: Some(AccountId::from([0x0; 32])),
                id,
            });
            Ok(())
        }

        /// Transfer token from caller.
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, id: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &to, &id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            id: u32,
        ) -> Result<(), Error> {
            self.transfer_token_from(&from, &to, &id)?;
            Ok(())
        }

        /// Sets or unsets the approval of a given operator to transfer all tokens of caller
        #[ink(message)]
        fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> bool {
            let caller = self.env().caller();
            if to == caller {
                return false;
            }
            self.operator_approvals.insert((caller, to), approved);
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved,
            });
            true
        }

        /// Get whether an operator is approved by a given owner
        #[ink(message)]
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.is_approved_for_all(owner, operator)
        }

        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self
                .operator_approvals
                .get(&(owner, operator))
                .unwrap_or(&false)
        }

        // Private functions

        /// Transfers token `id` `from` the sender to the `to` AccountId.
        fn transfer_token_from(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            id: &TokenId,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };
            if !self.approved_or_owner(Some(caller), *id) {
                return Err(Error::NotApproved);
            };

            self.clear_approval(id)?;
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                id: *id,
            });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(
            &mut self,
            from: &AccountId,
            id: &TokenId,
        ) -> Result<(), Error> {
            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };

            self.decrease_counter_of(from)?;
            self.token_owner.remove(id).ok_or(Error::CanNotRemove)?;
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: &TokenId) -> Result<(), Error> {
            if self.exists(id) {
                return Err(Error::TokenExists);
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::ZeroAccountNotAllowed);
            };

            self.increase_counter_of(to)?;
            if !self.token_owner.insert(*id, *to).is_none() {
                return Err(Error::CanNotInsert);
            }
            Ok(())
        }

        /// Approve the passed AccountId to transfer the specified token
        /// on behalf of the message's sender.
        fn approve(&mut self, to: &AccountId, id: &TokenId) -> Result<(), Error> {
            let caller = self.env().caller();
            if self.owner_of(*id) != Some(caller) {
                return Err(Error::NotOwner);
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::ZeroAccountNotAllowed);
            };

            if !self.token_approvals.insert(*id, *to).is_none() {
                return Err(Error::CanNotInsert);
            };
            self.env().emit_event(Approval {
                from: caller,
                to: *to,
                id: *id,
            });
            Ok(())
        }

        /// Increase token counter from the `of` AccountId.
        fn increase_counter_of(&mut self, of: &AccountId) -> Result<(), Error> {
            if self.balance_of_or_zero(of) > 0 {
                let count = self
                    .owned_tokens_count
                    .get_mut(of)
                    .ok_or(Error::CanNotGetCounter)?;
                *count += 1;
                return Ok(());
            } else {
                match self.owned_tokens_count.insert(*of, 1) {
                    Some(_) => Err(Error::CanNotInsert),
                    None => Ok(()),
                }
            }
        }

        /// Decrease token counter from the `of` AccountId.
        fn decrease_counter_of(&mut self, of: &AccountId) -> Result<(), Error> {
            let count = self
                .owned_tokens_count
                .get_mut(of)
                .ok_or(Error::CanNotGetCounter)?;
            *count -= 1;
            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: &TokenId) -> Result<(), Error> {
            if !self.token_approvals.contains_key(id) {
                return Ok(());
            };

            match self.token_approvals.remove(id) {
                Some(_res) => Ok(()),
                None => Err(Error::CanNotRemove),
            }
        }

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u32 {
            *self.owned_tokens_count.get(of).unwrap_or(&0)
        }

        /// Returns true if the AccountId `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            from != Some(AccountId::from([0x0; 32]))
                && (from == self.owner_of(id) || from == self.approved_for(id))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: &TokenId) -> bool {
            self.token_owner.get(id).is_some() && self.token_owner.contains_key(id)
        }
    }
}
