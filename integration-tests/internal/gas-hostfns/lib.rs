#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod gas_hostfns {
    #[ink(storage)]
    pub struct GasHostfns {}

    impl GasHostfns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Checks that the host function `gas_limit` works
        #[ink(message)]
        pub fn gas_limit(&self) -> u64 {
            self.env().gas_limit()
        }

        /// Checks that the host function `gas_price` works
        #[ink(message)]
        pub fn gas_price(&self) -> u64 {
            self.env().gas_price()
        }

        /// Checks that the host function `ref_time_left` works
        #[ink(message)]
        pub fn ref_time_left(&self) -> u64 {
            self.env().ref_time_left()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_gas_limit_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("gas_hostfns", &ink_e2e::alice(), &mut GasHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<GasHostfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.gas_limit())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_gas_price_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("gas_hostfns", &ink_e2e::alice(), &mut GasHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<GasHostfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.gas_price())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_ref_time_left_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("gas_hostfns", &ink_e2e::alice(), &mut GasHostfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<GasHostfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.ref_time_left())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > 0);

            Ok(())
        }
    }
}
