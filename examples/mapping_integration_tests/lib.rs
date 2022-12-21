//! A smart contract which demonstrates functionality of `Mapping` functions.

#![cfg_attr(not(feature = "std"), no_std)]

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
        /// Returns the balance of a account, or `None` if the account is not in the `Mapping`.
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
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn insert_and_get_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract_id = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::alice(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(1_000));
            let size = client
                .call(&ink_e2e::alice(), insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            // then
            let get = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.get_balance());
            let balance = client
                .call(&ink_e2e::alice(), get, 0, None)
                .await
                .expect("Calling `get_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            assert!(size.is_none());
            assert_eq!(balance, Some(1_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_contains_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract_id = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(1_000));
            let _ = client
                .call(&ink_e2e::bob(), insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.");

            // then
            let contains = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.contains_balance());
            let is_there = client
                .call(&ink_e2e::bob(), contains, 0, None)
                .await
                .expect("Calling `contains_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            assert!(is_there);

            Ok(())
        }

        #[ink_e2e::test]
        async fn reinsert_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract_id = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::charlie(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let first_insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(1_000));
            let _ = client
                .call(&ink_e2e::charlie(), first_insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.");

            let insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(10_000));
            let size = client
                .call(&ink_e2e::charlie(), insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            // then
            assert!(size.is_some());

            let get = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.get_balance());
            let balance = client
                .call(&ink_e2e::charlie(), get, 0, None)
                .await
                .expect("Calling `get_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            assert_eq!(balance, Some(10_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_remove_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract_id = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::dave(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(3_000));
            let _ = client
                .call(&ink_e2e::dave(), insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            let remove = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.remove_balance());
            let _ = client
                .call(&ink_e2e::dave(), remove, 0, None)
                .await
                .expect("Calling `remove_balance` failed");

            // then
            let get = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.get_balance());
            let balance = client
                .call(&ink_e2e::dave(), get, 0, None)
                .await
                .expect("Calling `get_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            assert_eq!(balance, None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_take_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = MappingsRef::new();
            let contract_id = client
                .instantiate(
                    "mapping-integration-tests",
                    &ink_e2e::eve(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.insert_balance(4_000));
            let _ = client
                .call(&ink_e2e::eve(), insert, 0, None)
                .await
                .expect("Calling `insert_balance` failed")
                .value
                .expect("Input is valid, call must not fail.");

            let take = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.take_balance());
            let balance = client
                .call(&ink_e2e::eve(), take, 0, None)
                .await
                .expect("Calling `take_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            // then
            assert_eq!(balance, Some(4_000));

            let contains = ink_e2e::build_message::<MappingsRef>(contract_id)
                .call(|contract| contract.contains_balance());
            let is_there = client
                .call(&ink_e2e::eve(), contains, 0, None)
                .await
                .expect("Calling `contains_balance` failed")
                .value
                .expect("Input is valid, call must not fail.")
                .expect("Execution should not fail.");

            assert!(!is_there);

            Ok(())
        }
    }
}
