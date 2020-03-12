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
mod dns {
    use ink_core::storage;
    use scale;

    /// Emitted whenever a new name is being registered.
    #[ink(event)]
    struct Register {
        #[ink(topic)]
        name: Hash,
        #[ink(topic)]
        from: AccountId,
    }

    /// Emitted whenever an address changes.
    #[ink(event)]
    struct SetAddress {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_address: Option<AccountId>,
        #[ink(topic)]
        new_address: AccountId,
    }

    /// Emitted whenver a name is being transferred.
    #[ink(event)]
    struct Transfer {
        #[ink(topic)]
        name: Hash,
        from: AccountId,
        #[ink(topic)]
        old_owner: Option<AccountId>,
        #[ink(topic)]
        new_owner: AccountId,
    }

    /// Domain name service contract inspired by ChainX's [blog post]
    /// (https://medium.com/@chainx_org/secure-and-decentralized-polkadot-domain-name-system-e06c35c2a48d).
    ///
    /// # Note
    ///
    /// This is a port from the blog post's ink! 1.0 based version of the contract
    /// to ink! 2.0.
    ///
    /// # Description
    ///
    /// The main function of this contract is domain name resolution which
    /// refers to the retrieval of numeric values corresponding to readable
    /// and easily memorable names such as “polka.dot” which can be used
    /// to facilitate transfers, voting and dapp-related operations instead
    /// of resorting to long IP addresses that are hard to remember.
    #[ink(storage)]
    struct DomainNameService {
        /// A hashmap to store all name to addresses mapping.
        name_to_address: storage::HashMap<Hash, AccountId>,
        /// A hashmap to store all name to owners mapping.
        name_to_owner: storage::HashMap<Hash, AccountId>,
        /// The default address.
        default_address: storage::Value<AccountId>,
    }

    /// Errors that can occure upon calling this contract.
    #[derive(scale::Encode, scale::Decode)]
    pub enum Error {
        /// Returned if the name already exists upon registration.
        NameAlreadyExists,
        /// Returned if caller is not owner while required to.
        CallerIsNotOwner,
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl DomainNameService {
        /// Creates a new domain name service contract.
        #[ink(constructor)]
        fn new(&mut self) {
            self.default_address.set(AccountId::from([0x00; 32]));
        }

        /// Register specific name with caller as owner.
        #[ink(message)]
        fn register(&mut self, name: Hash) -> Result<()> {
            let caller = self.env().caller();
            if self.is_name_assigned(name) {
                return Err(Error::NameAlreadyExists)
            }
            self.name_to_owner.insert(name, caller);
            self.env().emit_event(Register { name, from: caller });
            Ok(())
        }

        /// Set address for specific name.
        #[ink(message)]
        fn set_address(&mut self, name: Hash, new_address: AccountId) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.get_owner_or_default(name);
            if caller != owner {
                return Err(Error::CallerIsNotOwner)
            }
            let old_address = self.name_to_address.insert(name, new_address);
            self.env().emit_event(SetAddress {
                name,
                from: caller,
                old_address,
                new_address,
            });
            Ok(())
        }

        /// Transfer owner to another address.
        #[ink(message)]
        fn transfer(&mut self, name: Hash, to: AccountId) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.get_owner_or_default(name);
            if caller != owner {
                return Err(Error::CallerIsNotOwner)
            }
            let old_owner = self.name_to_owner.insert(name, to);
            self.env().emit_event(Transfer {
                name,
                from: caller,
                old_owner,
                new_owner: to,
            });
            Ok(())
        }

        /// Get address for specific name.
        #[ink(message)]
        fn get_address(&self, name: Hash) -> AccountId {
            self.get_address_or_default(name)
        }

        /// Returns `true` if the name already assigned.
        #[ink(message)]
        fn is_name_assigned(&self, name: Hash) -> bool {
            self.name_to_owner.get(&name).is_some()
        }

        /// Returns the owner given the hash or the default address.
        fn get_owner_or_default(&self, name: Hash) -> AccountId {
            *self
                .name_to_owner
                .get(&name)
                .unwrap_or(&*self.default_address)
        }

        /// Returns the address given the hash or the default address.
        fn get_address_or_default(&self, name: Hash) -> AccountId {
            *self
                .name_to_address
                .get(&name)
                .unwrap_or(&*self.default_address)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
    }
}
