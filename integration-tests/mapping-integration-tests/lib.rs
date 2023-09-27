//! A smart contract which demonstrates functionality of `Mapping` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mapping_integration_tests {
    use ink::storage::Mapping;

    /// A contract for testing `Mapping` functionality.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Mappings {
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
    }

    impl Mappings {
        /// Demonstrates the usage of `Mappings::default()`
        ///
        /// Creates an empty mapping between accounts and balances.
        #[ink(constructor)]
        pub fn new() -> Self {
            let balances = Mapping::default();
            Self { balances }
        }

        /// Demonstrates the usage of `Mapping::get()`.
        ///
        /// Returns the balance of a account, or `None` if the account is not in the
        /// `Mapping`.
        #[ink(message)]
        pub fn get_balance(&self) -> Option<Balance> {
            let caller = Self::env().caller();
            self.balances.get(caller)
        }

        /// Demonstrates the usage of `Mappings::insert()`.
        ///
        /// Assigns the value to a given account.
        ///
        /// Returns the size of the pre-existing balance at the specified key if any.
        /// Returns `None` if the account was not previously in the `Mapping`.
        #[ink(message)]
        pub fn insert_balance(&mut self, value: Balance) -> Option<u32> {
            let caller = Self::env().caller();
            self.balances.insert(caller, &value)
        }

        /// Demonstrates the usage of `Mappings::size()`.
        ///
        /// Returns the size of the pre-existing balance at the specified key if any.
        /// Returns `None` if the account was not previously in the `Mapping`.
        #[ink(message)]
        pub fn size_balance(&mut self) -> Option<u32> {
            let caller = Self::env().caller();
            self.balances.size(caller)
        }

        /// Demonstrates the usage of `Mapping::contains()`.
        ///
        /// Returns `true` if the account has any balance assigned to it.
        #[ink(message)]
        pub fn contains_balance(&self) -> bool {
            let caller = Self::env().caller();
            self.balances.contains(caller)
        }

        /// Demonstrates the usage of `Mappings::remove()`.
        ///
        /// Removes the balance entry for a given account.
        #[ink(message)]
        pub fn remove_balance(&mut self) {
            let caller = Self::env().caller();
            self.balances.remove(caller);
        }

        /// Demonstrates the usage of `Mappings::take()`.
        ///
        /// Returns the balance of a given account removing it from storage.
        ///
        /// Returns `None` if the account is not in the `Mapping`.
        #[ink(message)]
        pub fn take_balance(&mut self) -> Option<Balance> {
            let caller = Self::env().caller();
            self.balances.take(caller)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn insert_and_get_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::alice(),
                    constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Mappings>();

            // when
            let insert = call.insert_balance(1_000);
            let size = client
                .call(&ink_e2e::alice(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            let get = call.get_balance();
            let balance = client
                .call(&ink_e2e::alice(), &get)
                .submit_dry_run()
                .await
                .return_value();

            assert!(size.is_none());
            assert_eq!(balance, Some(1_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_contains_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::bob(),
                    constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Mappings>();

            // when
            let insert = call.insert_balance(1_000);
            let _ = client
                .call(&ink_e2e::bob(), &insert)
                .submit_dry_run()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            let contains = call.contains_balance();
            let is_there = client
                .call(&ink_e2e::bob(), &contains)
                .submit_dry_run()
                .await
                .return_value();

            assert!(is_there);

            Ok(())
        }

        #[ink_e2e::test]
        async fn reinsert_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::charlie(),
                    constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Mappings>();

            // when
            let first_insert = call.insert_balance(1_000);
            let _ = client
                .call(&ink_e2e::charlie(), &first_insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let insert = call.insert_balance(10_000);
            let size = client
                .call(&ink_e2e::charlie(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            assert!(size.is_some());

            let get = call.get_balance();
            let balance = client
                .call(&ink_e2e::charlie(), &get)
                .submit_dry_run()
                .await
                .return_value();

            assert_eq!(balance, Some(10_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_remove_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::dave(),
                    constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Mappings>();

            // when
            let insert = call.insert_balance(3_000);
            let _ = client
                .call(&ink_e2e::dave(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let remove = call.remove_balance();
            let _ = client
                .call(&ink_e2e::dave(), &remove)
                .submit()
                .await
                .expect("Calling `remove_balance` failed");

            // then
            let get = call.get_balance();
            let balance = client
                .call(&ink_e2e::dave(), &get)
                .submit_dry_run()
                .await
                .return_value();

            assert_eq!(balance, None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_take_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::eve(),
                    constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<Mappings>();

            // when
            let insert = call.insert_balance(4_000);
            let _ = client
                .call(&ink_e2e::eve(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let take = call.take_balance();
            let balance = client
                .call(&ink_e2e::eve(), &take)
                .submit()
                .await
                .expect("Calling `take_balance` failed")
                .return_value();

            // then
            assert_eq!(balance, Some(4_000));

            let contains = call.contains_balance();
            let is_there = client
                .call(&ink_e2e::eve(), &contains)
                .submit_dry_run()
                .await
                .return_value();

            assert!(!is_there);

            Ok(())
        }
    }
}
