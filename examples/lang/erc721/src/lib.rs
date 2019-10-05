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

use ink_core::{
    env::DefaultSrmlTypes,
    memory::format,
    storage,
};
use ink_lang::contract;
use ink_model::EnvHandler;
use ink_core::storage::alloc::Initialize as _;
use ink_core::storage::alloc::AllocateUsing as _;
use std::ops::Index;

pub type Result<T, E> = core::result::Result<T, E>;

contract! {
    #![env = DefaultSrmlTypes]

    // Event deposited when a token transfer occurs
    event Transfer {
        from: AccountId,
        to: AccountId,
        token_id: u32,
    }

    // Event deposited when an approval occurs
    event Approval {
        owner: AccountId,
        to: AccountId,
        token_id: u32,
    }

    /// The storage items for a typical ERC721 token implementation.
    struct Erc721 {
        /// The total supply.
        token_owner: storage::HashMap<u32, AccountId>,
        token_approvals: storage::HashMap<u32, AccountId>,
        owned_tokens: storage::HashMap<AccountId, storage::Vec<u32>>,
        all_tokens: storage::Vec<u32>,
        name: storage::Vec<u8>,
        symbol: storage::Vec<u8>,

    }

    impl Deploy for Erc721 {
        fn deploy(&mut self, name: u8, symbol: u8) {
            self.name.push(name);
            self.symbol.push(symbol);
        }
    }

    impl Erc721 {

        pub(external) fn get_total_supply(&self) -> u32 {
            let total_supply = self.all_tokens.len();
            env.println(&format!("Erc20::total_supply = {:?}", total_supply));
            total_supply
        }
        
        pub(external) fn get_balance(&self, owner: AccountId) -> u32 {
            let balance = self.balance_of(&owner);
            env.println(&format!("Erc721::balance_of(owner = {:?}) = {:?}", owner, balance));
            balance
        }

        pub(external) fn get_owner(&self, token_id: u32) -> AccountId {
            let owner = self.owner_of(&token_id);
            env.println(&format!("Erc721::owner_of(token = {:?}) = {:?}", token_id, owner));
            owner
        }

    }

    impl Erc721 {
        
        fn approve(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, to: &AccountId, token_id: &u32) -> Result<(), &'static str> {
            let caller = env.caller();

            if caller == self.owner_of(token_id){

                self.token_approvals.insert(*token_id, caller);

                env.emit(Approval {
                    owner: caller,
                    to: *to,
                    token_id: *token_id,
                });

                Ok(())

            } else{
                Err("not owner")
            }
        }

        fn transfer_from(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, from: &AccountId, to: &AccountId, token_id: &u32) -> Result<(), &'static str> {        
            let caller = env.caller();

            if self.approved_or_owner(&caller, token_id){

                self.clear_approval(from, token_id)?;
                self.remove_token_from(from, token_id)?;
                self.add_token_to(env, to, token_id)?;

                env.emit(Transfer {
                    from: *from,
                    to: *to,
                    token_id: *token_id,
                });

                Ok(())

            } else{
                Err("not approved")
            }
            
        }

        fn clear_approval(&mut self, from: &AccountId, token_id: &u32) -> Result<(), &'static str> {
            
            if *from == self.owner_of(token_id){
                match self.token_approvals.remove(token_id) {
                    Some(_res) => Ok(()),
                    None => Err("cannot remove token approval")
                }
            } else{
                Err("not owner")
            }
        }

        fn remove_token_from(&mut self, from: &AccountId, token_id: &u32) -> Result<(), &'static str> {

            if self.owner_of(token_id) != *from {
                return Err("not owner")
            };

            if !self.owned_tokens.contains_key(from){
                return Err("tokens not found");
            };

            let tokens = self.owned_tokens
            .get_mut(from)
            .expect("cannot get tokens");

            match tokens.swap_remove(*tokens.index(*token_id)) {
                Some(_res) => Ok(()),
                None => Err("cannot remove token")
            }
        }

        fn add_token_to(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, to: &AccountId, token_id: &u32) -> Result<(), &'static str> {

            if self.owner_of(token_id) != AccountId::from([0x0; 32]){
                return Err("already assigned")
            };

            if !self.owned_tokens.contains_key(to){

                let new_vec = unsafe {storage::Vec::<u32>::allocate_using(&mut env.dyn_alloc)}.initialize_into(());
                
                self.owned_tokens
                    .insert(*to, new_vec)
                    .ok_or("cannot add tokens")?;
            };

            let tokens = self.owned_tokens
                .get_mut(to)
                .expect("cannot find tokens");

            tokens.push(*token_id);

            Ok(())
        }

        fn balance_of(&self, of: &AccountId, ) -> u32 {
            
            let balance = match self.owned_tokens.get(of) {
                Some(num) => num.len(),
                None => 0u32,
            };
            balance
        }

        fn owner_of(&self, token_id: &u32, ) -> AccountId {
            *self.token_owner.get(token_id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_for(&self, token_id: &u32) -> AccountId {
            *self.token_approvals.get(token_id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_or_owner(&self, owner: &AccountId, token_id: &u32) -> bool {
            *owner == self.owner_of(token_id) ||  *owner == self.approved_for(token_id)
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;

    #[test]
    fn deployment_works() {
        let alice = AccountId::from([0x0; 32]);
        env::test::set_caller::<Types>(alice);

        let erc721 = Erc721::deploy_mock(1,2);
        assert_eq!(erc721.get_total_supply(), 0);
        assert_eq!(erc721.get_balance(alice), 0);
        assert_eq!(erc721.get_owner(1), AccountId::from([0x0; 32]));
    }
}
