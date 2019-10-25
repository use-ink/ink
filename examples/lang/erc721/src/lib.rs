// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

use core::result::Result;
use ink_core::{
    env::DefaultSrmlTypes,
    memory::format,
    storage,
};
use ink_lang::contract;
use ink_model;
use scale::{
    Decode,
    Encode,
};

pub type EnvHandler = ink_model::EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>;
pub type TokenId = u32;
pub type Counter = u32;

#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

#[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum Error {
    NotOwner,
    NotApproved,
    TokenExists,
    TokenNotFound,
    CanNotInsert,
    CanNotRemove,
    CanNotGetCounter,
    AccountZeroNotAllowed,
}

impl Error {
    fn to_u32(self) -> u32 {
        self as u32
    }
}

contract! {
    #![env = DefaultSrmlTypes]

    /// Event deposited when a token transfer occurs
    event Transfer {
        from: AccountId,
        to: AccountId,
        id: TokenId,
    }

    /// Event deposited when an approval occurs
    event Approval {
        owner: AccountId,
        to: AccountId,
        id: TokenId,
    }

    /// The storage items for a typical ERC721 token implementation.
    struct Erc721 {
        token_owner: storage::HashMap<TokenId, AccountId>,
        token_approvals: storage::HashMap<TokenId, AccountId>,
        owned_tokens_count: storage::HashMap<AccountId, Counter>,
    }

    impl Deploy for Erc721 {
        fn deploy(&mut self) {}
    }

    impl Erc721 {
        /// External wrapper for `fn balance_of()`.
        pub(external) fn get_balance(&self, owner: AccountId) -> u32 {
            let balance = self.balance_of(&owner);
            env.println(&format!("Erc721::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        /// External wrapper for `fn owner_of()`.
        pub(external) fn get_owner(&self, id: u32) -> AccountId {
            let owner = self.owner_of(&id);
            env.println(&format!("Erc721::owner_of(token = {:?}) = {:?}", id, owner));
            owner
        }

        /// External wrapper for `fn mint()`.
        pub(external) fn mint_token(&mut self, to: AccountId, id: u32) -> Result<(), u32> {
            self.mint(env, &to, &id)?;
            env.println(&format!("Erc721::minted(token = {:?}) = {:?}", id, to));
            Ok(())
        }

        /// External wrapper for `fn transfer_from()`.
        pub(external) fn transfer_token(&mut self, from: AccountId, to: AccountId, id: u32) -> Result<(), u32> {
            self.transfer_from(env, &from, &to, &id)?;
            env.println(&format!("Erc721::transferred(token = {:?}) = {:?}", id, to));
            Ok(())
        }

        /// External wrapper for `fn burn()`.
        pub(external) fn burn_token(&mut self, from: AccountId, id: u32) -> Result<(), u32> {
            self.burn(env, &from, &id)?;
            env.println(&format!("Erc721::burned(token = {:?}) = {:?}", id, from));
            Ok(())
        }

        /// External wrapper for `fn approve()`.
        pub(external) fn approve_transfer(&mut self, to: AccountId, id: u32) -> Result<(), u32> {
            self.approve(env, &to, &id)?;
            env.println(&format!("Erc721::approved(token = {:?}) = {:?}", id, to));
            Ok(())
        }
    }

    impl Erc721 {
        /// Transfers token `id` `from` the sender to the `to` AccountId.
        fn transfer_from(&mut self, env: &EnvHandler, from: &AccountId, to: &AccountId, id: &TokenId) -> Result<(), u32> {
            let caller = env.caller();
            if !self.exists(id) {
                return Err(Error::TokenNotFound.to_u32())
            };
            if !self.approved_or_owner(&caller, id){
                return Err(Error::NotApproved.to_u32())
            };

            self.clear_approval(id)?;
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            env.emit(Transfer {
                from: *from,
                to: *to,
                id: *id,
            });
            Ok(())
        }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: &TokenId) -> Result<(), u32> {
            if !self.token_approvals.contains_key(id) {
                return Ok(())
            };

            match self.token_approvals.remove(id) {
                Some(_res) => Ok(()),
                None => Err(Error::CanNotRemove.to_u32())
            }
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(&mut self, from: &AccountId, id: &TokenId) -> Result<(), u32> {
            if !self.exists(id) {
                return Err(Error::TokenNotFound.to_u32());
            };

            self.decrease_counter_of(from)?;
            self.token_owner
                .remove(id)
                .ok_or(Error::CanNotRemove.to_u32())?;
            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: &TokenId) -> Result<(), u32> {
            if self.exists(id) {
                 return Err(Error::TokenExists.to_u32())
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::AccountZeroNotAllowed.to_u32())
            };

            self.increase_counter_of(to)?;
            if !self.token_owner.insert(*id, *to).is_none() {
                return Err(Error::CanNotInsert.to_u32())
            }
            Ok(())
        }

        /// Approve the passed AccountId to transfer the specified token
        /// on behalf of the message's sender.
        fn approve(&mut self, env: &EnvHandler, to: &AccountId, id: &TokenId) -> Result<(), u32> {
            let caller = env.caller();
            if self.owner_of(id) != caller {
                return Err(Error::NotOwner.to_u32())
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::AccountZeroNotAllowed.to_u32())
            };

            if !self.token_approvals.insert(*id, *to).is_none() {
                return Err(Error::CanNotInsert.to_u32())
            };
            env.emit(Approval {
                owner: caller,
                to: *to,
                id: *id,
            });
            Ok(())
        }

        /// Creates a  unique token `id` assigned to the `to` AccountId.
        fn mint(&mut self, env: &EnvHandler, to: &AccountId, id: &TokenId) -> Result<(), u32> {
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::AccountZeroNotAllowed.to_u32())
            };
            if self.exists(id) {
                return Err(Error::TokenExists.to_u32())
            }

            if !self.token_owner.insert(*id, *to).is_none() {
                return Err(Error::CanNotInsert.to_u32())
            }
            self.increase_counter_of(to)?;
            env.emit(Transfer {
                from: AccountId::from([0x0; 32]),
                to: *to,
                id: *id,
            });
            Ok(())
        }

        /// Destroys an existing token `id` owned by the caller.
        fn burn(&mut self, env: &EnvHandler, from: &AccountId, id: &TokenId)-> Result<(), u32> {
            let caller = env.caller();
            if !self.exists(id) {
                 return Err(Error::TokenNotFound.to_u32())
            };
            if self.owner_of(id) != caller && *from != AccountId::from([0x0; 32]) {
                return Err(Error::NotOwner.to_u32())
            };

            self.clear_approval(id)?;
            self.decrease_counter_of(from)?;
            self.token_owner
                .remove(id)
                .ok_or(Error::CanNotRemove.to_u32())?;
            env.emit(Transfer {
                from: *from,
                to: AccountId::from([0x0; 32]),
                id: *id,
            });
            Ok(())
        }

        /// Increase token counter from the `of` AccountId.
        fn increase_counter_of(&mut self, of: &AccountId) -> Result<(), u32> {
            if self.balance_of(of) > 0 {
                let count = self.owned_tokens_count
                    .get_mut(of)
                    .ok_or(Error::CanNotGetCounter.to_u32())?;
                *count += 1;
                return Ok(())
            } else {
                match self.owned_tokens_count.insert(*of, 1) {
                    Some(_) => Err(Error::CanNotInsert.to_u32()),
                    None => Ok(()),
                }
            }
        }

        /// Decrease token counter from the `of` AccountId.
        fn decrease_counter_of(&mut self, of: &AccountId) -> Result<(), u32> {
            let count = self.owned_tokens_count
                .get_mut(of)
                .ok_or(Error::CanNotGetCounter.to_u32())?;
            *count -= 1;
            Ok(())
        }

        /// Returns the total number of tokens from an account.
        fn balance_of(&self, of: &AccountId) -> u32 {
            *self.owned_tokens_count.get(of).unwrap_or(&0u32)
        }

        /// Returns the owner of a token or AccountId 0x0 if it does not exists.
        fn owner_of(&self, id: &TokenId) -> AccountId {
            *self.token_owner.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        /// Returns the approved AccountId froma token `id`
        /// or AccountId 0x0 if it does not exists.
        fn approved_for(&self, id: &TokenId) -> AccountId {
            *self.token_approvals.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        /// Returns true if the AccountId `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: &AccountId, id: &TokenId) -> bool {
            *from != AccountId::from([0x0; 32]) &&
            (*from == self.owner_of(id) ||  *from == self.approved_for(id))
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: &TokenId) -> bool {
            self.token_owner.get(id).is_some() && self.token_owner.contains_key(id)
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;
    type Erc721Test = test::TestableErc721;

    /// Generate testing accounts.
    fn generate_accounts(length: u8) -> Option<Vec<AccountId>> {
        if length > 0 {
            let mut accounts: Vec<AccountId> = vec![AccountId::from([0x0; 32]); 1];
            for n in 1..=length {
                accounts.push(AccountId::from([n; 32]));
            }
            Some(accounts)
        } else {
            None
        }
    }

    /// Deploys an ERC721 test contract.
    fn initialize_erc721(from: AccountId) -> Erc721Test {
        env::test::set_caller::<Types>(from);
        Erc721::deploy_mock()
    }

    #[test]
    fn deployment_works() {
        let accounts = generate_accounts(3).unwrap();
        let erc721 = initialize_erc721(accounts[0]);
        // AccountId 0 can not owns tokens.
        assert_eq!(erc721.get_balance(accounts[0]), 0);
        // Token 0 does not exists.
        assert_eq!(erc721.get_owner(0), accounts[0]);
    }

    #[test]
    fn mint_works() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Token 1 does not exists.
        assert_eq!(erc721.get_owner(1), accounts[0]);
        // AccountId 1 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 1 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
        // Create token Id 2 owned by AccountId 1.
        assert_eq!(erc721.mint_token(accounts[1], 2), Ok(()));
        // AccountId 1 owns 2 tokens.
        assert_eq!(erc721.get_balance(accounts[1]), 2);
        // Token Id 2 is owned by AccountId 1.
        assert_eq!(erc721.get_owner(2), accounts[1]);
        // Create token Id 3.
        assert_eq!(erc721.mint_token(accounts[2], 3), Ok(()));
        // Account Id 2 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        // Token Id 3 is owned by AccountId 2.
        assert_eq!(erc721.get_owner(3), accounts[2]);
    }

    #[test]
    fn mint_existing_should_fail() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 1 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
        // Cannot create  token Id if it exists.
        // AccountId 2 cannot own token Id 1.
        assert_eq!(
            erc721.mint_token(accounts[2], 1),
            Err(Error::TokenExists.to_u32())
        );
        // AccountId 2 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        // AccountId 1 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
    }

    #[test]
    fn transfer_works() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 1 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
        // Change transaction caller to AccountId 1.
        env::test::set_caller::<Types>(accounts[1]);
        // Transfer token Id 1 from AccountId 1 to AccountId 2.
        assert_eq!(erc721.transfer_token(accounts[1], accounts[2], 1), Ok(()));
        // AccountId 1 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        // AccountId 2 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        // AccountId 2 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[2]);
    }

    #[test]
    fn invalid_transfer_should_fail() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Transfer token fails if it does not exists.
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[2], 2),
            Err(Error::TokenNotFound.to_u32())
        );
        // Token Id 2 does not exists.
        assert_eq!(erc721.get_owner(2), accounts[0]);
        // Create token Id 2.
        assert_eq!(erc721.mint_token(accounts[1], 2), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // Token Id 2 is owned by AccountId 1.
        assert_eq!(erc721.get_owner(2), accounts[1]);
        // Change transaction caller to AccountId 2.
        env::test::set_caller::<Types>(accounts[2]);
        // AccountId 2 cannot transfer not owned tokens.
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[2], 2),
            Err(Error::NotApproved.to_u32())
        );
    }

    #[test]
    fn burn_works() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 1 owns token Id 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
        // Change transaction caller to AccountId 1.
        env::test::set_caller::<Types>(accounts[1]);
        // Transfer Token Id 1 from AccountId 1 to AccountId 2.
        assert_eq!(erc721.transfer_token(accounts[1], accounts[2], 1), Ok(()));
        // AccountId 1 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        // AccountId 2 owns token 1.
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        // Token Id 1 is owned by AccountId 2.
        assert_eq!(erc721.get_owner(1), accounts[2]);
        // Change transaction caller to AccountId 2.
        env::test::set_caller::<Types>(accounts[2]);
        // Destroy token Id 1.
        assert_eq!(erc721.burn_token(accounts[2], 1), Ok(()));
        // AccountId 2 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[2]), 0);
    }

    #[test]
    fn approved_transfer_works() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // Token Id 1 is owned by AccountId 1.
        assert_eq!(erc721.get_owner(1), accounts[1]);
        // Change transaction caller to AccountId 1.
        env::test::set_caller::<Types>(accounts[1]);
        // Approve token Id 1 transfer for AccountId 2 on behalf of the owner.
        assert_eq!(erc721.approve_transfer(accounts[2], 1), Ok(()));
        // Change transaction caller to AccountId 2.
        env::test::set_caller::<Types>(accounts[2]);
        // Transfer token Id 1 from AccountId 1 to AccountId 3.
        assert_eq!(erc721.transfer_token(accounts[1], accounts[3], 1), Ok(()));
        // TokenId 3 is owned by AccountId 3.
        assert_eq!(erc721.get_owner(1), accounts[3]);
        // AccountId 1 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        // AccountId 2 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        // AccountId 3 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[3]), 1);
    }

    #[test]
    fn not_approved_transfer_should_fail() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        // Create token Id 1.
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 2 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        // AccountId 3 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[3]), 0);
        // Change transaction caller to AccountId 2.
        env::test::set_caller::<Types>(accounts[2]);
        // AccountId 3 is not approved by AccountId 1.
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[3], 1),
            Err(Error::NotApproved.to_u32())
        );
        // AccountId 1 owns 1 token.
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        // AccountId 2 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        // AccountId 3 does not owns tokens.
        assert_eq!(erc721.get_balance(accounts[3]), 0);
    }
}
