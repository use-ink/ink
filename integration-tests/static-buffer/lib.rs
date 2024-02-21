#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod static_buffer {
    use ink::prelude::vec::Vec;

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

        #[ink(message)]
        pub fn buffer(&self) {
            let buf1 = Vec::<u8>::with_capacity(3);
            let buf2 = Vec::<u64>::with_capacity(1);
            ink::env::debug_println!("{:?}", buf1.as_ptr());
            ink::env::debug_println!("{:?}", buf2.as_ptr());
            ink::env::debug_println!("{}", core::mem::align_of::<Vec<bool>>());
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
            let call_builder = contract.call_builder::<StaticBuffer>();

            // when
            let get = call_builder.get_caller();
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

        #[ink_e2e::test]
        async fn buffer<Client: E2EBackend>(mut client: Client) -> E2EResult<()> {
            // given
            let mut constructor = StaticBufferRef::new_default();

            // when
            let contract = client
                .instantiate("static_buffer", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<StaticBuffer>();

            // then
            let get = call_builder.buffer();
            let get_res = client.call(&ink_e2e::bob(), &get).submit().await?;

            let debug_msg = get_res.debug_message();
            let msgs: Vec<&str> = debug_msg.split('\n').collect();
            let ptr1 = u64::from_str_radix(msgs[0].trim_start_matches("0x"), 16).unwrap();
            let ptr2 = u64::from_str_radix(msgs[1].trim_start_matches("0x"), 16).unwrap();
            let align = u64::from_str_radix(msgs[2], 10).unwrap();

            assert_eq!(align, 4);
            assert_eq!((ptr2 - ptr1), 8);

            Ok(())
        }
    }
}
