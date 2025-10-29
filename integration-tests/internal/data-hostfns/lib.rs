#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod data_hostfns {
    #[ink(storage)]
    pub struct DataHostfns {}

    impl DataHostfns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Checks that the host function `call_data_size` works
        #[ink(message)]
        pub fn call_data_size(&self) -> u64 {
            self.env().call_data_size()
        }

        /// Checks that the host function `return_data_size` works
        #[ink(message)]
        pub fn return_data_size(&self) -> u64 {
            self.env().return_data_size()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_call_data_size_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("data_hostfns", &ink_e2e::alice(), &mut DataHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DataHostfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.call_data_size())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_return_data_size_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("data_hostfns", &ink_e2e::alice(), &mut DataHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<DataHostfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.return_data_size())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() == 0); // no calls were made, thus is 0

            Ok(())
        }
    }
}
