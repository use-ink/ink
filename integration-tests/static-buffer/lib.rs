#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod static_buffer {
    #[allow(unused_imports)]
    use ink::env::BUFFER_SIZE;
    #[ink(storage)]
    pub struct StaticBuffer {
        value: bool,
    }

    impl StaticBuffer {
        /// Creates a dummy smart contract.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Sets a default value.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Returns the caller of the contract.
        /// Should panic if the buffer size is less than 32 bytes.
        #[ink(message)]
        pub fn get_caller(&self) -> AccountId {
            self.env().caller()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        #[should_panic(expected = "the output buffer is too small!")]
        fn run_out_buffer_memory() {
            let flipper = StaticBuffer::new(false);
            flipper.get_caller()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_run_out_of_buffer_memory<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let mut constructor = StaticBufferRef::new(false);
            let contract = client
                .instantiate("static_buffer", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call = contract.call::<StaticBuffer>();

            // when
            let get = call.get_caller();
            // then panics if `INK_STATIC_BUFFER_SIZE` is less than 32 bytes.
            let res = client.call(&ink_e2e::bob(), &get).dry_run().await;
            println!("{}", super::BUFFER_SIZE);
            assert!(
                res.is_err(),
                "Buffer size was larger than expected: {}",
                super::BUFFER_SIZE.to_string()
            );

            Ok(())
        }
    }
}
