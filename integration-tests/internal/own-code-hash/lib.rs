#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod own_code_hash {
    use ink::H256;

    #[ink(storage)]
    pub struct OwnCodeHash {}

    impl OwnCodeHash {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the code hash of the contract
        #[ink(message)]
        pub fn own_code_hash(&self) -> H256 {
            self.env().own_code_hash().unwrap()
        }

        /// Returns the code hash of the contract by providing its `account_id`
        #[ink(message)]
        pub fn get_code(&self) -> H256 {
            self.env()
                .code_hash(&self.env().address())
                .expect("Failed to get code hash")
        }
    }

    impl Default for OwnCodeHash {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn get_own_code_hash() {
            let code_hash = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                OwnCodeHashRef,
            >();
            let address = {
                let create_params = ink::env::call::build_create::<OwnCodeHashRef>()
                    .code_hash(code_hash)
                    .endowment(0.into())
                    .exec_input(ink::env::call::ExecutionInput::new(
                        ink::env::call::Selector::new(ink::selector_bytes!("new")),
                    ))
                    .returns::<OwnCodeHashRef>()
                    .params();

                let cr = ink::env::instantiate_contract(&create_params)
                    .unwrap_or_else(|error| {
                        panic!(
                            "Received an error from `pallet-revive` while instantiating: {error:?}"
                        )
                    })
                    .unwrap_or_else(|error| {
                        panic!("Received a `LangError` while instantiating: {error:?}")
                    });
                ink::ToAddr::to_addr(&cr)
            };

            let own_code_hash = OwnCodeHash::new();
            ink::env::test::set_callee(address);
            let code_hash_via_own: H256 = own_code_hash.own_code_hash();

            assert_eq!(code_hash_via_own, code_hash);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink_e2e::ContractsBackend;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn get_own_code_hash(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let mut constructor = OwnCodeHashRef::new();
            let contract = client
                .instantiate("own_code_hash", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");

            let call_builder = contract.call_builder::<OwnCodeHash>();
            let own_code_hash_res = client
                .call(&ink_e2e::bob(), &call_builder.own_code_hash())
                .submit()
                .await
                .expect("own_code_hash failed");

            // Compare codes obtained differently with own_code_hash and code_hash
            let get_code_res = client
                .call(&ink_e2e::alice(), &call_builder.get_code())
                .submit()
                .await
                .expect("get_code failed");

            let code_hash_via_own = own_code_hash_res.return_value();
            let code_hash_via_get = get_code_res.return_value();

            assert_eq!(code_hash_via_own, code_hash_via_get);

            Ok(())
        }
    }
}
