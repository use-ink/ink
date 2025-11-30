#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod storage_hostfns {
    use ink::U256;

    #[ink(storage)]
    pub struct Storagefns {}

    impl Storagefns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Sets a value in persistent storage.
        #[ink(message)]
        pub fn set_storage(&self, key: U256, value: [u8; 32]) -> Option<u32> {
            self.env().set_storage(key, &value)
        }

        /// Sets a value in transient storage.
        #[ink(message)]
        pub fn set_transient_storage(&self, key: U256, value: [u8; 32]) -> Option<u32> {
            self.env().set_transient_storage(key, &value)
        }

        /// Clears a persistent storage entry by setting value to all zeros.
        #[ink(message)]
        pub fn clear_storage(&self, key: U256) -> Option<u32> {
            self.env().set_storage(key, &[0u8; 32])
        }

        /// Clears a transient storage entry by setting value to all zeros.
        #[ink(message)]
        pub fn set_clear_transient_storage(&self, key: U256, value: [u8; 32]) -> Option<u32> {
            self.env().set_transient_storage(key, &value);
            self.env().set_transient_storage(key, &[0u8; 32])
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn set_storage_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StoragefnsRef::new();
            let contract = client
                .instantiate("storage-hostfns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Storagefns>();

            // when - set a value
            let key = U256::from(42u32);
            let value = [0xDEu8; 32];
            let result = client
                .call(&ink_e2e::alice(), &call_builder.set_storage(key, value))
                .submit()
                .await?;

            // then - first set returns None (no previous value)
            assert_eq!(result.return_value(), None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn clear_storage_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StoragefnsRef::new();
            let contract = client
                .instantiate("storage-hostfns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Storagefns>();

            let key = U256::from(42u32);
            let value = [0xDEu8; 32];

            // when - set a value
            let _ = client
                .call(&ink_e2e::alice(), &call_builder.set_storage(key, value))
                .submit()
                .await?;

            // when - clear it
            let result = client
                .call(&ink_e2e::alice(), &call_builder.clear_storage(key))
                .submit()
                .await?;

            // then - should return size of previous value (32 bytes)
            assert_eq!(result.return_value(), Some(32u32));

            Ok(())
        }

        #[ink_e2e::test]
        async fn set_transient_storage_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StoragefnsRef::new();
            let contract = client
                .instantiate("storage-hostfns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Storagefns>();

            // when - set a transient value
            let key = U256::from(50u32);
            let value = [0xABu8; 32];
            let result = client
                .call(&ink_e2e::alice(), &call_builder.set_transient_storage(key, value))
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            // then - first set returns None
            assert_eq!(result.return_value(), None);

            Ok(())
        }

        #[ink_e2e::test]
        async fn clear_transient_storage_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StoragefnsRef::new();
            let contract = client
                .instantiate("storage-hostfns", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Storagefns>();

            let key = U256::from(50u32);
            let value = [0x1u8; 32];

            let result = client
                .call(&ink_e2e::alice(), &call_builder.set_clear_transient_storage(key, value))
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            // then - should return size of previous value (32 bytes)
            assert_eq!(result.return_value(), Some(32u32));

            Ok(())
        }
    }
}
