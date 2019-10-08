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
    storage::Flush,
};
use ink_lang::contract;
use ink_model::EnvHandler;
use scale::{Encode, Decode};

pub type Result<T, E> = core::result::Result<T, E>;
pub type TokenId = u32;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
struct Counter {
    value: storage::Value<u32>,
}

impl Counter {
    pub fn increase(&mut self) {
        self.value += 1;
    }

    pub fn decrease(&mut self) {
        self.value -= 1;
    }

    pub fn get(&self) -> u32 {
        *self.value
    }
}

impl Flush for Counter {
	fn flush(&mut self) {}
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
        owned_tokens_count: storage::HashMap<AccountId, Counter>,
        all_tokens: storage::Vec<TokenId>,
    }

    impl Deploy for Erc721 {
        fn deploy(&mut self) {}
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

        pub(external) fn get_owner(&self, id: u32) -> AccountId {
            let owner = self.owner_of(&id);
            env.println(&format!("Erc721::owner_of(token = {:?}) = {:?}", id, owner));
            owner
        }
    }

    impl Erc721 {
        fn approve(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, to: &AccountId, id: &TokenId) -> Result<(), &'static str> {
            let caller = env.caller();
            if caller == self.owner_of(id){
                self.token_approvals
                    .insert(*id, caller)
                    .ok_or("cannot approve token")?;
                env.emit(Approval {
                    owner: caller,
                    to: *to,
                    id: *id,
                });
                Ok(())
            } else{
                Err("not owner")
            }
        }

        fn transfer_from(&mut self, env: &mut EnvHandler<ink_core::env::ContractEnv<DefaultSrmlTypes>>, from: &AccountId, to: &AccountId, id: &TokenId) -> Result<(), &'static str> {        
            let caller = env.caller();
            if !self.approved_or_owner(&caller, id){
                return Err("not approved")
            };

            self.clear_approval(from, id)?;
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;
            env.emit(Transfer {
                from: *from,
                to: *to,
                id: *id,
            });
            Ok(())
        }

        fn clear_approval(&mut self, from: &AccountId, id: &TokenId) -> Result<(), &'static str> {
            if self.owner_of(id) != *from {
                return Err("not owner")
            };
            if !self.token_approvals.contains_key(id){
                return Err("approval not found");
            };

            match self.token_approvals.remove(id) {
                Some(_res) => Ok(()),
                None => Err("cannot remove token approval")
            }
        }

        fn remove_token_from(&mut self, from: &AccountId, id: &TokenId) -> Result<(), &'static str> {
            if self.owner_of(id) != *from {
                return Err("not owner")
            };
            if !self.token_owner.contains_key(id){
                return Err("token not found");
            };

            match self.token_owner.remove(id) {
                Some(_res) => Ok(()),
                None => Err("cannot remove token")
            };
            let count = self.owned_tokens_count
                .get_mut(from)
                .ok_or("cannot get account count")?;
            count.decrease();
            Ok(())
        }

        fn add_token_to(&mut self, to: &AccountId, id: &TokenId) -> Result<(), &'static str> {
            if self.owner_of(id) != AccountId::from([0x0; 32]){
                return Err("already assigned")
            };

            if !self.token_owner.contains_key(id){
                self.token_owner
                    .insert(*id, *to)
                    .ok_or("cannot insert token")?;
            let count = self.owned_tokens_count
                .get_mut(to)
                .ok_or("cannot get account count")?;
            count.increase();
            Ok(())
            } else{
                Err("token has owner")
            }

        }

        fn balance_of(&self, of: &AccountId) -> u32 {
            let balance: u32 = match self.owned_tokens_count.get(of) {
                Some(count) => count.get(),
                None => 0u32,
            };
            balance
        }

        fn owner_of(&self, id: &TokenId, ) -> AccountId {
            *self.token_owner.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_for(&self, id: &TokenId) -> AccountId {
            *self.token_approvals.get(id).unwrap_or(&AccountId::from([0x0; 32]))
        }

        fn approved_or_owner(&self, owner: &AccountId, id: &TokenId) -> bool {
            *owner == self.owner_of(id) ||  *owner == self.approved_for(id)
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

        let erc721 = Erc721::deploy_mock();
        assert_eq!(erc721.get_total_supply(), 0);
        assert_eq!(erc721.get_balance(alice), 0);
        assert_eq!(erc721.get_owner(1), AccountId::from([0x0; 32]));
    }
}
