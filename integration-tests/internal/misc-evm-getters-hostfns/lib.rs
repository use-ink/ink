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

        /// Checks that the host function `balance_of` works
        #[ink(message)]
        pub fn balance_of(&self) -> U256 {
            let caller = self.env().caller();
            self.env().balance_of(caller)
        }

        /// Checks that the host function `base_fee` works
        #[ink(message)]
        pub fn base_fee(&self) -> U256 {
            self.env().base_fee()
        }

        /// Checks that the host function `origin` works
        #[ink(message)]
        pub fn origin(&self) -> Address {
            self.env().origin()
        }

        /// Checks that the host function `code_size` works
        #[ink(message)]
        pub fn code_size(&self) -> u64 {
            let this_addr = self.env().address();
            self.env().code_size(this_addr)
        }

        /// Checks that the host function `block_author` works
        #[ink(message)]
        pub fn block_author(&self) -> Address {
            self.env().block_author()
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::{ContractsBackend,address_from_keypair};

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

        #[ink_e2e::test]
        async fn e2e_balance_of_works<Client: E2EBackend>(
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
                .call(&ink_e2e::alice(), &call_builder.balance_of())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > U256::from(0));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_base_fee_works<Client: E2EBackend>(
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
                .call(&ink_e2e::alice(), &call_builder.base_fee())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > U256::from(0));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_origin_works<Client: E2EBackend>(
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
                .call(&ink_e2e::alice(), &call_builder.origin())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert_eq!(call_res.return_value(), address_from_keypair::<AccountId>(&ink_e2e::alice()));

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_code_size_works<Client: E2EBackend>(
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
                .call(&ink_e2e::alice(), &call_builder.code_size())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            assert!(call_res.return_value() > 0);

            Ok(())
        }

        #[ink_e2e::test]
        async fn e2e_block_author_works<Client: E2EBackend>(
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
            let _call_res = client
                .call(&ink_e2e::alice(), &call_builder.block_author())
                .submit()
                .await
                .unwrap_or_else(|err| {
                    panic!("call failed: {:#?}", err);
                });

            Ok(())
        }
    }
}
