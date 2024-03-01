//! A smart contract which demonstrates functionality of `Mapping` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mapping_integration_tests {
    use ink::{
        prelude::{
            string::String,
            vec::Vec,
        },
        storage::Mapping,
    };

    #[derive(Debug, PartialEq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum ContractError {
        ValueTooLarge,
    }

    /// A contract for testing `Mapping` functionality.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Mappings {
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
        /// Mapping from owner to aliases.
        names: Mapping<AccountId, Vec<String>>,
    }

    impl Mappings {
        /// Demonstrates the usage of `Mappings::default()`
        ///
        /// Creates an empty mapping between accounts and balances.
        #[ink(constructor)]
        pub fn new() -> Self {
            let balances = Mapping::default();
            let names = Mapping::default();
            Self { balances, names }
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

        /// Demonstrates the usage of `Mappings::try_take()` and `Mappings::try_insert()`.
        ///
        /// Adds a name of a given account.
        ///
        /// Returns `Ok(None)` if the account is not in the `Mapping`.
        /// Returns `Ok(Some(_))` if the account was already in the `Mapping`
        /// Returns `Err(_)` if the mapping value couldn't be encoded.
        #[ink(message)]
        pub fn try_insert_name(&mut self, name: String) -> Result<(), ContractError> {
            let caller = Self::env().caller();
            let mut names = match self.names.try_take(caller) {
                None => Vec::new(),
                Some(value) => value.map_err(|_| ContractError::ValueTooLarge)?,
            };

            names.push(name);

            self.names
                .try_insert(caller, &names)
                .map_err(|_| ContractError::ValueTooLarge)?;

            Ok(())
        }

        /// Demonstrates the usage of `Mappings::try_get()`.
        ///
        /// Returns the name of a given account.
        ///
        /// Returns `Ok(None)` if the account is not in the `Mapping`.
        /// Returns `Ok(Some(_))` if the account was already in the `Mapping`
        /// Returns `Err(_)` if the mapping value couldn't be encoded.
        #[ink(message)]
        pub fn try_get_names(&mut self) -> Option<Result<Vec<String>, ContractError>> {
            let caller = Self::env().caller();
            self.names
                .try_get(caller)
                .map(|result| result.map_err(|_| ContractError::ValueTooLarge))
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
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when
            let insert = call_builder.insert_balance(1_000);
            let size = client
                .call(&ink_e2e::alice(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            let get = call_builder.get_balance();
            let balance = client
                .call(&ink_e2e::alice(), &get)
                .dry_run()
                .await?
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
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::bob(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when
            let insert = call_builder.insert_balance(1_000);
            let _ = client
                .call(&ink_e2e::bob(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            let contains = call_builder.contains_balance();
            let is_there = client
                .call(&ink_e2e::bob(), &contains)
                .dry_run()
                .await?
                .return_value();

            assert!(is_there);

            Ok(())
        }

        #[ink_e2e::test]
        async fn reinsert_works<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::charlie(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when
            let first_insert = call_builder.insert_balance(1_000);
            let _ = client
                .call(&ink_e2e::charlie(), &first_insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let insert = call_builder.insert_balance(10_000);
            let size = client
                .call(&ink_e2e::charlie(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            // then
            assert!(size.is_some());

            let get = call_builder.get_balance();
            let balance = client
                .call(&ink_e2e::charlie(), &get)
                .dry_run()
                .await?
                .return_value();

            assert_eq!(balance, Some(10_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_remove_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::dave(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when
            let insert = call_builder.insert_balance(3_000);
            let _ = client
                .call(&ink_e2e::dave(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let remove = call_builder.remove_balance();
            let _ = client
                .call(&ink_e2e::dave(), &remove)
                .submit()
                .await
                .expect("Calling `remove_balance` failed");

            // then
            let get = call_builder.get_balance();
            let balance = client
                .call(&ink_e2e::dave(), &get)
                .dry_run()
                .await?
                .return_value();

            assert_eq!(balance, None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_take_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::eve(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when
            let insert = call_builder.insert_balance(4_000);
            let _ = client
                .call(&ink_e2e::eve(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed")
                .return_value();

            let take = call_builder.take_balance();
            let balance = client
                .call(&ink_e2e::eve(), &take)
                .submit()
                .await
                .expect("Calling `take_balance` failed")
                .return_value();

            // then
            assert_eq!(balance, Some(4_000));

            let contains = call_builder.contains_balance();
            let is_there = client
                .call(&ink_e2e::eve(), &contains)
                .dry_run()
                .await?
                .return_value();

            assert!(!is_there);

            Ok(())
        }

        #[ink_e2e::test]
        async fn fallible_storage_methods_work<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = MappingsRef::new();
            let contract = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::ferdie(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Mappings>();

            // when the mapping value overgrows the buffer
            let name = ink_e2e::ferdie().public_key().to_account_id().to_string();
            let insert = call_builder.try_insert_name(name.clone());
            let mut names = Vec::new();
            while let Ok(_) = client.call(&ink_e2e::ferdie(), &insert).submit().await {
                names.push(name.clone())
            }

            // then adding another one should fail gracefully
            let expected_insert_result = client
                .call(&ink_e2e::ferdie(), &insert)
                .dry_run()
                .await?
                .return_value();
            let received_insert_result =
                Err(crate::mapping_integration_tests::ContractError::ValueTooLarge);
            assert_eq!(received_insert_result, expected_insert_result);

            // then there should be 4 entries (that's what fits into the 256kb buffer)
            let received_mapping_value = client
                .call(&ink_e2e::ferdie(), &call_builder.try_get_names())
                .dry_run()
                .await?
                .return_value();
            let expected_mapping_value = Some(Ok(names));
            assert_eq!(received_mapping_value, expected_mapping_value);

            Ok(())
        }
    }
}
