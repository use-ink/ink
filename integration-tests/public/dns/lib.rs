#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dns {
    use ink::{
        H256,
        storage::Mapping,
    };

    /// Emitted whenever a new name is being registered.
    #[ink(event)]
    pub struct Register {
        #[ink(topic)]
        name: H256,
        #[ink(topic)]
        from: Address,
    }

    /// Emitted whenever an address changes.
    #[ink(event)]
    pub struct SetAddress {
        #[ink(topic)]
        name: H256,
        from: Address,
        #[ink(topic)]
        old_address: Option<Address>,
        #[ink(topic)]
        new_address: Address,
    }

    /// Emitted whenever a name is being transferred.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        name: H256,
        from: Address,
        #[ink(topic)]
        old_owner: Option<Address>,
        #[ink(topic)]
        new_owner: Address,
    }

    /// Domain name service contract inspired by
    /// [this blog post](https://medium.com/@chainx_org/secure-and-decentralized-polkadot-domain-name-system-e06c35c2a48d).
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
    /// and easily memorable names such as "polka.dot" which can be used
    /// to facilitate transfers, voting and DApp-related operations instead
    /// of resorting to long IP addresses that are hard to remember.
    #[ink(storage)]
    pub struct DomainNameService {
        /// A hashmap to store all name to addresses mapping.
        name_to_address: Mapping<H256, Address>,
        /// A hashmap to store all name to owners mapping.
        name_to_owner: Mapping<H256, Address>,
        /// The default address.
        default_address: Address,
    }

    impl Default for DomainNameService {
        fn default() -> Self {
            let mut name_to_address = Mapping::new();
            name_to_address.insert(H256::default(), &zero_address());
            let mut name_to_owner = Mapping::new();
            name_to_owner.insert(H256::default(), &zero_address());

            Self {
                name_to_address,
                name_to_owner,
                default_address: zero_address(),
            }
        }
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
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
        pub fn new() -> Self {
            Default::default()
        }

        /// Register specific name with caller as owner.
        #[ink(message)]
        pub fn register(&mut self, name: H256) -> Result<()> {
            let caller = self.env().caller();
            if self.name_to_owner.contains(name) {
                return Err(Error::NameAlreadyExists)
            }

            self.name_to_owner.insert(name, &caller);
            self.env().emit_event(Register { name, from: caller });

            Ok(())
        }

        /// Set address for specific name.
        #[ink(message)]
        pub fn set_address(&mut self, name: H256, new_address: Address) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.get_owner_or_default(name);
            if caller != owner {
                return Err(Error::CallerIsNotOwner)
            }

            let old_address = self.name_to_address.get(name);
            self.name_to_address.insert(name, &new_address);

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
        pub fn transfer(&mut self, name: H256, to: Address) -> Result<()> {
            let caller = self.env().caller();
            let owner = self.get_owner_or_default(name);
            if caller != owner {
                return Err(Error::CallerIsNotOwner)
            }

            let old_owner = self.name_to_owner.get(name);
            self.name_to_owner.insert(name, &to);

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
        pub fn get_address(&self, name: H256) -> Address {
            self.get_address_or_default(name)
        }

        /// Get owner of specific name.
        #[ink(message)]
        pub fn get_owner(&self, name: H256) -> Address {
            self.get_owner_or_default(name)
        }

        /// Returns the owner given the hash or the default address.
        fn get_owner_or_default(&self, name: H256) -> Address {
            self.name_to_owner.get(name).unwrap_or(self.default_address)
        }

        /// Returns the address given the hash or the default address.
        fn get_address_or_default(&self, name: H256) -> Address {
            self.name_to_address
                .get(name)
                .unwrap_or(self.default_address)
        }
    }

    /// Helper for referencing the zero address (`0x00`). Note that in practice this
    /// address should not be treated in any special way (such as a default
    /// placeholder) since it has a known private key.
    fn zero_address() -> Address {
        [0u8; 20].into()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn default_accounts() -> ink::env::test::DefaultAccounts {
            ink::env::test::default_accounts()
        }

        fn set_next_caller(caller: Address) {
            ink::env::test::set_caller(caller);
        }

        #[ink::test]
        fn register_works() {
            let default_accounts = default_accounts();
            let name = H256::from([0x99; 32]);

            set_next_caller(default_accounts.alice);
            let mut contract = DomainNameService::new();

            assert_eq!(contract.register(name), Ok(()));
            assert_eq!(contract.register(name), Err(Error::NameAlreadyExists));
        }

        #[ink::test]
        fn set_address_works() {
            let accounts = default_accounts();
            let name = H256::from([0x99; 32]);

            set_next_caller(accounts.alice);

            let mut contract = DomainNameService::new();
            assert_eq!(contract.register(name), Ok(()));

            // Caller is not owner, `set_address` should fail.
            set_next_caller(accounts.bob);
            assert_eq!(
                contract.set_address(name, accounts.bob),
                Err(Error::CallerIsNotOwner)
            );

            // Caller is owner, set_address will be successful
            set_next_caller(accounts.alice);
            assert_eq!(contract.set_address(name, accounts.bob), Ok(()));
            assert_eq!(contract.get_address(name), accounts.bob);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts = default_accounts();
            let name = H256::from([0x99; 32]);

            set_next_caller(accounts.alice);

            let mut contract = DomainNameService::new();
            assert_eq!(contract.register(name), Ok(()));

            // Test transfer of owner.
            assert_eq!(contract.transfer(name, accounts.bob), Ok(()));

            // Owner is bob, alice `set_address` should fail.
            assert_eq!(
                contract.set_address(name, accounts.bob),
                Err(Error::CallerIsNotOwner)
            );

            set_next_caller(accounts.bob);
            // Now owner is bob, `set_address` should be successful.
            assert_eq!(contract.set_address(name, accounts.bob), Ok(()));
            assert_eq!(contract.get_address(name), accounts.bob);
        }
    }
}
