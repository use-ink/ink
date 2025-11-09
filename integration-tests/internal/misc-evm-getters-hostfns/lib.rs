#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
mod misc_evm_getters_hostfns {
    use ink::U256;

    #[ink(storage)]
    pub struct MiscEVMGettersfns {}

    impl MiscEVMGettersfns {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Checks that the host function `chain_id` works
        #[ink(message)]
        pub fn chain_id(&self) -> U256 {
            self.env().chain_id()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn e2e_chain_id_works<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            // given
            let contract = client
                .instantiate("misc_evm_getters_hostfns", &ink_e2e::alice(), &mut MiscEVMGettersfnsRef::new())
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<MiscEVMGettersfns>();

            // then
            let call_res = client
                .call(&ink_e2e::alice(), &call_builder.chain_id())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > U256::from(0));

            Ok(())
        }
    }
}
