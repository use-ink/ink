#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod static_buffer {
    use ink::prelude::{
        string::String,
        vec::Vec,
    };

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
        pub fn buffer(&self) -> Result<(u64, u64), String> {
            let buf1 = Vec::<u8>::with_capacity(3);
            let buf2 = Vec::<u64>::with_capacity(1);
            let ptr1 = buf1.as_ptr() as u64;
            let ptr2 = buf2.as_ptr() as u64;
            let align = core::mem::align_of::<Vec<u64>>() as u64;
            let padding = ptr2
                .checked_sub(ptr1)
                .ok_or(String::from("Error during padding calculation"))?;
            Ok((padding, align))
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
            let buffer_call = call_builder.buffer();
            let buffer_call_res =
                client.call(&ink_e2e::bob(), &buffer_call).submit().await?;
            let value = buffer_call_res.return_value();
            assert!(value.is_ok());
            let value = value.unwrap();
            let padding = value.0;
            let align = value.1;
            assert_eq!(padding, 8);
            assert_eq!(align, 4);
            Ok(())
        }
    }
}
