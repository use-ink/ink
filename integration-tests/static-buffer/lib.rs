#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod static_buffer {
    #[ink(storage)]
    pub struct StaticBuffer {
        value: bool,
    }

    impl StaticBuffer {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Returns the configured size of the static buffer.
        #[ink(message)]
        pub fn get_size(&self) -> u128 {
            ink::env::STATIC_BUFFER_SIZE as u128
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn off_chain_buffer_size() {
            let instance = StaticBuffer::new_default();
            let expected_size: u128 = 1 << 19; // 512 kB
            assert_eq!(instance.get_size(), expected_size);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn on_chain_buffer_size<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let constructor = StaticBufferRef::new(false);
            let contract = client
                .instantiate("static_buffer", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed");
            let call = contract.call::<StaticBuffer>();

            // then
            let get = call.get_size();
            let get_res = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            let expected_size: u128 = 1 << 19; // 512 kB
            assert_eq!(get_res.return_value(), expected_size);

            Ok(())
        }
    }
}
