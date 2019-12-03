// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang2 as ink;
use ink_core::storage;

#[ink::contract(version = "0.1.0")]
mod erc721 {
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
        operator_approvals: storage::HashMap<(AccountId, AccountId), bool>
    }

    /// Event emitted when a token transfer occurs
    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: AccountId,
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
        approved: bool
    }

    impl Erc721 {
        #[ink(constructor)]
        fn new(&mut self) {

        }

        /// ===========================
        /// NOT REQUIRED, JUST FOR TEST
        /// ===========================
        #[ink(message)]
        fn mint(&mut self, id: TokenId) -> bool {
            let caller = self.env().caller();
            let owner = self.owner_of(id);
            match owner {
                None => false,
                Some(_owner) => {
                    self.token_owner.insert(id, caller);
                    let balance = self.balance_of_or_zero(caller);
                    self.owned_tokens_count.insert(caller, balance + 1);
                    self.env().emit_event(Transfer {
                        from: None,
                        to: caller,
                        id: id,
                    });
                    true
                }
            }
        }

        /// Get token balance of specific account.
        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u32 {
            let balance = self.balance_of_or_zero(owner);
            balance
        }

        /// Get owner for specific token.
        #[ink(message)]
        fn owner_of(&self, id: TokenId) -> Option<AccountId> {
            self.token_owner.get(&id).cloned()
        }

        /// The approved address for this token, or the none address if there is none
        #[ink(message)]
        fn get_approved(&self, id: TokenId) -> Option<AccountId> {
            self.approved_of_or_none(id)
        }

        /// Sets or unsets the approval of a given operator to transfer all tokens of caller
        #[ink(message)]
        fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> bool {
            let caller = self.env().caller();
            if to == caller {
                return false
            }
            self.operator_approvals.insert((caller, to), approved);
            self.env().emit_event(ApprovalForAll {
                owner: caller,
                operator: to,
                approved
            });
            true
        }

        /// Get whether an operator is approved by a given owner
        #[ink(message)]
        fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.is_approved_for_all_impl(owner, operator)
        }

        /// Transfer token from owner to another address
        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, id: TokenId) -> bool {
            let caller = self.env().caller();
            if self.is_approved_or_owner(caller, id) {
                return self.transfer_from_impl(from, to, id);
            }
            false
        }

        /// Approve another account to operate the given token
        #[ink(message)]
        fn approve(&mut self, to: AccountId, id: TokenId) -> bool {
            let caller = self.env().caller();
            let owner = self.owner_of(id);
            match owner {
                None => false,
                Some(owner) => {
                    if caller != owner {
                        return false;
                    }
                    if owner == to {
                        return false;
                    }
                    self.token_approvals.insert(id, to);
                    self.env().emit_event(Approval {
                        from: owner,
                        to: to,
                        id: id,
                    });
                    true
                }
            }
        }

        // Private functions

        fn balance_of_or_zero(&self, of: AccountId) -> u32 {
            *self.owned_tokens_count.get(&of).unwrap_or(&0)
        }

        fn approved_of_or_none(&self, id: TokenId) -> Option<AccountId> {
            self.token_approvals.get(&id).cloned()
        }

        fn is_approved_or_owner(&self, spender: AccountId, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            match owner {
                None => return false,
                Some(owner) => {
                    if spender == owner {
                        return true
                    } else {
                        if self.is_approved_for_all_impl(owner, spender) {
                            return true
                        }
                    }
                }
            }
            let approved_account = self.approved_of_or_none(id);
            match approved_account {
                None => {},
                Some(account) => {
                    if spender == account {
                        return true;
                    }
                }
            }
            false
        }

        fn is_approved_for_all_impl(&self, owner: AccountId, operator: AccountId) -> bool {
            *self.operator_approvals.get(&(owner, operator)).unwrap_or(&false)
        }

        fn transfer_from_impl(&mut self, from: AccountId, to: AccountId, id: TokenId) -> bool {

            self.clear_approval(id);

            let from_balance = self.balance_of_or_zero(from);
            let to_balance = self.balance_of_or_zero(to);

            self.owned_tokens_count.insert(from, from_balance - 1);
            self.owned_tokens_count.insert(to, to_balance + 1);
            self.token_owner.insert(id, to);

            true
        }

        fn clear_approval(&mut self, id: TokenId) {
            self.token_approvals.remove(&id);
        }
    }
}

