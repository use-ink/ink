#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod mappings {
    use ink::storage::Mapping;

    /// A simple ERC-20 contract.
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
        /// Returns the balance of a account.
        ///
        /// Returns `None` if the account is non-existent.
        #[ink(message)]
        pub fn get_balance(&self) -> Option<Balance> {
            let caller = Self::env().caller();
            let value = self.balances.get(caller);
            ink::env::debug_println!("Balance: {:?}", value);
            value
        }

        /// Demonstrates the usage of `Mappings::insert()`.
        ///
        /// Assigns the value to a given account.
        /// Returns the size of the pre-existing balance at the specified key if any.
        ///
        /// Returns `None` if the account was non-existent.
        #[ink(message)]
        pub fn insert_balance(&mut self, value: Balance) -> Option<u32> {
            let caller = Self::env().caller();
            self.balances.insert(caller, &value)
        }

        /// Demonstrates the usage of `Mappings::size()`.
        ///
        /// Returns the size of the pre-existing value at the specified key if any.
        ///
        /// Returns `None` if the account was non-existent.
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
        /// Returns the balance of a given account
        /// removing it from storage.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn take_balance(&mut self) -> Option<Balance> {
            let caller = Self::env().caller();
            self.balances.take(caller)
        }
    }

    #[cfg(test)]
    mod e2e_tests {
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn insert_and_get_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = mappings::constructors::new();
            let contract_id = client
                .instantiate(&mut ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = mappings::messages::insert_balance(1_000);
            let size = client
                .call(&mut ink_e2e::alice(), contract_id.clone(), insert, 0, None)
                .await
                .expect("call failed")
                .value;

            // then
            let get = mappings::messages::get_balance();
            let balance = client
                .call(&mut ink_e2e::alice(), contract_id.clone(), get, 0, None)
                .await
                .expect("call failed")
                .value;

            assert!(size.is_none());
            assert_eq!(balance, Some(1_000));

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_contains_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = mappings::constructors::new();
            let contract_id = client
                .instantiate(&mut ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let insert = mappings::messages::insert_balance(2_000);
            let _ = client
                .call(&mut ink_e2e::bob(), contract_id.clone(), insert, 0, None)
                .await
                .expect("call failed");

            // then
            let contains = mappings::messages::contains_balance();
            let is_there = client
                .call(&mut ink_e2e::bob(), contract_id.clone(), contains, 0, None)
                .await
                .expect("call failed")
                .value;

            assert!(is_there);

            Ok(())
        }

        #[ink_e2e::test]
        async fn reinsert_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // given
            let constructor = mappings::constructors::new();
            let contract_id = client
                .instantiate(&mut ink_e2e::charlie(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let first_insert = mappings::messages::insert_balance(1_000_000);
            let _ = client
                .call(
                    &mut ink_e2e::charlie(),
                    contract_id.clone(),
                    first_insert,
                    0,
                    None,
                )
                .await
                .expect("call failed");

            let insert = mappings::messages::insert_balance(10_000);
            let size = client
                .call(
                    &mut ink_e2e::charlie(),
                    contract_id.clone(),
                    insert,
                    0,
                    None,
                )
                .await
                .expect("call failed")
                .value;

            // then
            assert!(size.is_some());

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_remove_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = mappings::constructors::new();
            let contract_id = client
                .instantiate(&mut ink_e2e::dave(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = mappings::messages::insert_balance(3_000);
            let _ = client
                .call(&mut ink_e2e::dave(), contract_id.clone(), insert, 0, None)
                .await
                .expect("call failed");

            let remove = mappings::messages::remove_balance();
            let _ = client
                .call(&mut ink_e2e::dave(), contract_id.clone(), remove, 0, None)
                .await
                .expect("call failed");

            // then
            let get = mappings::messages::get_balance();
            let balance = client
                .call(&mut ink_e2e::dave(), contract_id.clone(), get, 0, None)
                .await
                .expect("call failed")
                .value;

            assert_eq!(balance, None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn insert_and_take_works(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let constructor = mappings::constructors::new();
            let contract_id = client
                .instantiate(&mut ink_e2e::eve(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // when
            let insert = mappings::messages::insert_balance(4_000);
            let _ = client
                .call(&mut ink_e2e::eve(), contract_id.clone(), insert, 0, None)
                .await
                .expect("call failed");

            let take = mappings::messages::take_balance();
            let balance = client
                .call(&mut ink_e2e::eve(), contract_id.clone(), take, 0, None)
                .await
                .expect("call failed")
                .value;

            // then
            let contains = mappings::messages::contains_balance();
            let is_not_there = client
                .call(&mut ink_e2e::eve(), contract_id.clone(), contains, 0, None)
                .await
                .expect("call failed")
                .value;

            assert_eq!(balance, Some(4_000));
            assert!(!is_not_there);

            Ok(())
        }
    }
}
