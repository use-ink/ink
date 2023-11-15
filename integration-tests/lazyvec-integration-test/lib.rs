//! A smart contract which demonstrates functionality of `lazyvec` functions.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod lazyvec_integration_tests {

    #[ink(storage)]
    pub struct LazyVector {
        /// Stores indivudual bytes values on the storage.
        vec: ink::storage::StorageVec<u8>,
    }

    impl LazyVector {
        #[ink(constructor, payable)]
        pub fn default() -> Self {
            Self {
                vec: Default::default(),
            }
        }

        /// Push another byte value to storage.
        #[ink(message)]
        pub fn push(&mut self, value: u8) {
            self.vec.push(&value);
        }

        /// Pop the last byte value from storage (removing it from storage).
        #[ink(message)]
        pub fn pop(&mut self) -> Option<u8> {
            self.vec.pop()
        }

        /// Peek at the last byte value without removing it from storage.
        #[ink(message)]
        pub fn peek(&self, at: u32) -> Option<u8> {
            self.vec.get(at)
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
            let mut constructor = LazyVectorRef::default();
            let contract = client
                .instantiate(
                    "lazyvec-integration-tests",
                    &ink_e2e::alice(),
                    &mut constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let mut call = contract.call::<LazyVector>();

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
