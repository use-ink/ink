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

#[derive(Encode, Decode, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum ErrNo {
    NotOwner,
    NotApproved,
    InsertError,
    TokenExists,
    TokenNotFound,
    CanNotRemove,
    CanNotGetCounter,
    AccountZeroNotAllowed,
}

impl From<ErrNo> for u32 {
    fn from(item: ErrNo) -> Self {
        item as u32
    }
}

contract! {
    #![env = DefaultSrmlTypes]

    // Event deposited when a token transfer occurs
    event Transfer {
        from: AccountId,
        to: AccountId,
        id: TokenId,
    }

    // Event deposited when an approval occurs
    event Approval {
        owner: AccountId,
        to: AccountId,
        id: TokenId,
    }

    /// The storage items for a typical ERC721 token implementation.
    struct Erc721 {
        token_owner: storage::HashMap<TokenId, AccountId>,
        token_approvals: storage::HashMap<TokenId, AccountId>,
        owned_tokens_count: storage::HashMap<AccountId, Counter>
    }

    impl Deploy for Erc721 {
        fn deploy(&mut self) {}
    }

    impl Erc721 {
        pub(external) fn get_balance(&self, owner: AccountId) -> u32 {
            let balance = self.balance_of(&owner);
            env.println(&format!("Erc721::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        pub(external) fn get_owner(&self, id: u32) -> AccountId {
            let owner = self.owner_of(&id);
            env.println(&format!("Erc721::owner_of(token = {:?}) = {:?}", id, owner));
            owner
        }

        pub(external) fn mint_token(&mut self, to: AccountId, id: u32) -> Result<(), u32> {
            self.mint(env, &to, &id)?;
            env.println(&format!("Erc721::minted(token = {:?}) = {:?}", id, to));
            Ok(())
        }

        pub(external) fn transfer_token(&mut self, from: AccountId, to: AccountId, id: u32) -> Result<(), u32> {
            self.transfer_from(env, &from, &to, &id)?;
            env.println(&format!("Erc721::transferred(token = {:?}) = {:?}", id, to));
            Ok(())
        }

        pub(external) fn burn_token(&mut self, from: AccountId, id: u32) -> Result<(), u32> {
            self.burn(env, &from, &id)?;
            env.println(&format!("Erc721::burned(token = {:?}) = {:?}", id, from));
            Ok(())
        }

        pub(external) fn approve_transfer(&mut self, to: AccountId, id: u32) -> Result<(), u32> {
            self.approve(env, &to, &id)?;
            env.println(&format!("Erc721::approved(token = {:?}) = {:?}", id, to));
            Ok(())
        }
    }

    impl Erc721 {
        fn approve(&mut self, env: &EnvHandler, to: &AccountId, id: &TokenId) -> Result<(), ErrNo> {
            let caller = env.caller();
            if self.owner_of(id) != caller {
                return Err(ErrNo::NotOwner)
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(ErrNo::AccountZeroNotAllowed)
            };

            if !self.token_approvals.insert(*id, *to).is_none() {
                return Err(ErrNo::InsertError)
            };
            env.emit(Approval {
                owner: caller,
                to: *to,
                id: *id,
            });
            Ok(())
        }

        fn transfer_from(&mut self, env: &EnvHandler, from: &AccountId, to: &AccountId, id: &TokenId) -> Result<(), ErrNo> {
            let caller = env.caller();
            if !self.exists(id){
                return Err(ErrNo::TokenNotFound);
            };
            if !self.approved_or_owner(&caller, id){
                return Err(ErrNo::NotApproved)
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

        fn clear_approval(&mut self, id: &TokenId) -> Result<(), ErrNo> {
            if !self.token_approvals.contains_key(id){
                return Ok(());
            };

            match self.token_approvals.remove(id) {
                Some(_res) => Ok(()),
                None => Err(ErrNo::CanNotRemove)
            }
        }

        fn remove_token_from(&mut self, from: &AccountId, id: &TokenId) -> Result<(), ErrNo> {
            if !self.exists(id){
                return Err(ErrNo::TokenNotFound);
            };

            self.decrease_counter_of(from)?;
            self.token_owner
                .remove(id)
                .ok_or(ErrNo::CanNotRemove)?;
            Ok(())
        }

        fn add_token_to(&mut self, to: &AccountId, id: &TokenId) -> Result<(), ErrNo> {
            if self.exists(id){
                 return Err(ErrNo::TokenExists)
            };

            self.increase_counter_of(to)?;
            if !self.token_owner.insert(*id, *to).is_none() {
                return Err(ErrNo::InsertError)
            };
            Ok(())
        }

        fn mint(&mut self, env: &EnvHandler, to: &AccountId, id: &TokenId) -> Result<(), ErrNo> {
            if *to == AccountId::from([0x0; 32]){
                return Err(ErrNo::AccountZeroNotAllowed)
            };
            if self.exists(id){
                return Err(ErrNo::TokenExists)
            };

            if !self.token_owner.insert(*id, *to).is_none() {
                return Err(ErrNo::InsertError)
            };
            self.increase_counter_of(to)?;
            env.emit(Transfer {
                from: AccountId::from([0x0; 32]),
                to: *to,
                id: *id,
            });
            Ok(())
        }

        fn burn(&mut self, env: &EnvHandler, from: &AccountId, id: &TokenId)-> Result<(), ErrNo> {
            let caller = env.caller();
            if !self.exists(id){
                 return Err(ErrNo::TokenNotFound)
            };
            if self.owner_of(id) != caller && *from != AccountId::from([0x0; 32]) {
                return Err(ErrNo::NotOwner)
            };

            self.clear_approval(id)?;
            self.decrease_counter_of(from)?;
            self.token_owner
                .remove(id)
                .ok_or(ErrNo::CanNotRemove)?;
            env.emit(Transfer {
                from: *from,
                to: AccountId::from([0x0; 32]),
                id: *id,
            });
            Ok(())
        }

        fn increase_counter_of(&mut self, of: &AccountId) -> Result<(), ErrNo> {
            if self.balance_of(of) > 0 {
                let count = self.owned_tokens_count
                    .get_mut(of)
                    .ok_or(ErrNo::CanNotGetCounter)?;
                *count += 1;
                return Ok(());
            } else{
                match self.owned_tokens_count.insert(*of, 1) {
                    Some(_) => return Err(ErrNo::InsertError),
                    None => return Ok(())
                };
            };
        }

        fn decrease_counter_of(&mut self, of: &AccountId) -> Result<(), ErrNo> {
            let count = self.owned_tokens_count
                .get_mut(of)
                .ok_or(ErrNo::CanNotGetCounter)?;
            *count -= 1;
            Ok(())
        }

        fn balance_of(&self, of: &AccountId) -> u32 {
            *self.owned_tokens_count.get(of).unwrap_or(&0u32)
        }

        fn owner_of(&self, id: &TokenId, ) -> AccountId {
            *self.token_owner.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_for(&self, id: &TokenId) -> AccountId {
            *self.token_approvals.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_or_owner(&self, from: &AccountId, id: &TokenId) -> bool {
            *from != AccountId::from([0x0; 32]) &&
            (*from == self.owner_of(id) ||  *from == self.approved_for(id))
        }

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

    fn initialize_erc721(from: AccountId) -> Erc721Test {
        env::test::set_caller::<Types>(from);
        Erc721::deploy_mock()
    }

    #[test]
    fn deployment_works() {
        let accounts = generate_accounts(3).unwrap();
        let erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.get_balance(accounts[0]), 0);
        assert_eq!(erc721.get_owner(1), AccountId::from([0x0; 32]));
    }

    #[test]
    fn mint_works() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.get_owner(1), accounts[0]);
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_owner(1), accounts[1]);
        assert_eq!(erc721.mint_token(accounts[1], 2), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 2);
        assert_eq!(erc721.get_owner(2), accounts[1]);
        assert_eq!(erc721.mint_token(accounts[2], 3), Ok(()));
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        assert_eq!(erc721.get_owner(3), accounts[2]);
    }

    #[test]
    fn mint_existing_should_fail() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_owner(1), accounts[1]);
        assert_eq!(
            erc721.mint_token(accounts[2], 1),
            Err(ErrNo::TokenExists.into())
        );
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        assert_eq!(erc721.get_owner(1), accounts[1]);
    }

    #[test]
    fn transfer_works() {
        let accounts = generate_accounts(2).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_owner(1), accounts[1]);

        env::test::set_caller::<Types>(accounts[1]);
        assert_eq!(erc721.transfer_token(accounts[1], accounts[2], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        assert_eq!(erc721.get_owner(1), accounts[2]);
    }

    #[test]
    fn invalid_transfer_should_fail() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[2], 2),
            Err(ErrNo::TokenNotFound.into())
        );
        assert_eq!(erc721.get_owner(2), accounts[0]);
        assert_eq!(erc721.mint_token(accounts[1], 2), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_owner(2), accounts[1]);

        env::test::set_caller::<Types>(accounts[2]);
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[2], 2),
            Err(ErrNo::NotApproved.into())
        );
        assert_eq!(erc721.mint_token(accounts[2], 55), Ok(()));
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        assert_eq!(erc721.get_owner(55), accounts[2]);

        env::test::set_caller::<Types>(accounts[2]);
        assert_eq!(
            erc721.transfer_token(accounts[2], accounts[3], 2),
            Err(ErrNo::NotApproved.into())
        );
        assert_eq!(erc721.get_owner(2), accounts[1]);
    }

    #[test]
    fn burn_works() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_owner(1), accounts[1]);

        env::test::set_caller::<Types>(accounts[1]);
        assert_eq!(erc721.transfer_token(accounts[1], accounts[2], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        assert_eq!(erc721.get_balance(accounts[2]), 1);
        assert_eq!(erc721.get_owner(1), accounts[2]);

        env::test::set_caller::<Types>(accounts[2]);
        assert_eq!(erc721.burn_token(accounts[2], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[2]), 0);
    }

    #[test]
    fn approved_transfer_works() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);

        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_owner(1), accounts[1]);

        env::test::set_caller::<Types>(accounts[1]);
        assert_eq!(erc721.approve_transfer(accounts[2], 1), Ok(()));

        env::test::set_caller::<Types>(accounts[2]);
        assert_eq!(erc721.transfer_token(accounts[1], accounts[3], 1), Ok(()));
        assert_eq!(erc721.get_owner(1), accounts[3]);
        assert_eq!(erc721.get_balance(accounts[1]), 0);
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        assert_eq!(erc721.get_balance(accounts[3]), 1);
    }

    #[test]
    fn not_approved_transfer_should_fail() {
        let accounts = generate_accounts(3).unwrap();
        let mut erc721 = initialize_erc721(accounts[0]);
        assert_eq!(erc721.mint_token(accounts[1], 1), Ok(()));
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        assert_eq!(erc721.get_balance(accounts[3]), 0);

        env::test::set_caller::<Types>(accounts[2]);
        assert_eq!(
            erc721.transfer_token(accounts[1], accounts[3], 1),
            Err(ErrNo::NotApproved.into())
        );
        assert_eq!(erc721.get_balance(accounts[1]), 1);
        assert_eq!(erc721.get_balance(accounts[2]), 0);
        assert_eq!(erc721.get_balance(accounts[3]), 0);
    }
}
