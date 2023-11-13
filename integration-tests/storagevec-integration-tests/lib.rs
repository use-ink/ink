//! A smart contract which demonstrates functionality of `Mapping` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod storagevec_integration_tests {
    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct StorageVector {
        /// Stores a single `bool` value on the storage.
        vec: ink::storage::StorageVec<u8>,
    }

    impl StorageVector {
        /// Constructor that initializes the `bool` value to `false`.
        ///
        /// Constructors can delegate to other constructors.
        #[ink(constructor, payable)]
        pub fn default() -> Self {
            Self {
                vec: Default::default(),
            }
        }

        /// A message that can be called on instantiated contracts.
        /// This one flips the value of the stored `bool` from `true`
        /// to `false` and vice versa.
        #[ink(message)]
        pub fn push(&mut self, value: u8) {
            self.vec.push(value);
            self.vec.write();
        }

        /// Simply returns the current value of our `bool`.
        #[ink(message)]
        pub fn pop(&mut self) -> Option<u8> {
            let result = self.vec.pop();
            self.vec.write();
            result
        }

        #[ink(message)]
        pub fn peek(&self, at: u32) -> Option<u8> {
            self.vec.get(at).copied()
        }

        #[ink(message)]
        pub fn get(&self) -> ink::prelude::vec::Vec<u8> {
            self.vec.iter().copied().collect()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn push_and_pop_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StorageVectorRef::default();
            let contract = client
                .instantiate(
                    "storagevec-integration-tests",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<StorageVector>();

            // when
            let insert = call.push(0);
            let _ = client
                .call(&ink_e2e::alice(), &insert)
                .submit()
                .await
                .expect("Calling `insert_balance` failed");

            // then
            let value = client
                .call(&ink_e2e::alice(), &call.pop())
                .dry_run()
                .await
                .return_value();
            assert_eq!(value, Some(0));

            client
                .call(&ink_e2e::alice(), &call.pop())
                .submit()
                .await
                .unwrap();

            let value = client
                .call(&ink_e2e::alice(), &call.pop())
                .dry_run()
                .await
                .return_value();
            assert_eq!(value, None);

            Ok(())
        }
    }
}
