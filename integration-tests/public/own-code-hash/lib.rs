#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod own_code_hash {

    #[ink(storage)]
    pub struct OwnCodeHash {}

    impl OwnCodeHash {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Returns the code hash of the contract
        #[ink(message)]
        pub fn own_code_hash(&self) -> Hash {
            self.env().own_code_hash().unwrap()
        }

        /// Returns the code hash of the contract by providing it's `account_id`
        #[ink(message)]
        pub fn get_code(&self) -> Hash {
            self.env()
                .code_hash(&self.env().account_id())
                .expect("Failed to get code hash")
        }
    }

    impl Default for OwnCodeHash {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(all(test, feature = "test_instantiate"))]
    mod tests {
        use super::*;

        #[ink::test]
        fn get_own_code_hash() {
            let code_hash = ink::env::simulate_code_upload::<ink::env::DefaultEnvironment, OwnCodeHashRef>();
            let address = 
            {
                let create_params = ink::env::call::build_create::<OwnCodeHashRef>()
                    .code_hash(code_hash)
                    .gas_limit(0)
                    .endowment(0)
                    .exec_input(ink::env::call::ExecutionInput::new(
                        ink::env::call::Selector::new(ink::selector_bytes!("new")),
                    ))
                    .salt_bytes(&[0_u8; 4])
                    .returns::<OwnCodeHashRef>()
                    .params();

                let cr = ink::env::instantiate_contract(&create_params)
                    .unwrap_or_else(|error| {
                        panic!(
                            "Received an error from the Contracts pallet while instantiating: {:?}",
                            error
                        )
                    })
                    .unwrap_or_else(|error| {
                        panic!("Received a `LangError` while instatiating: {:?}", error)
                    });
                ink::ToAccountId::<ink::env::DefaultEnvironment>::to_account_id(&cr)
            };

            let own_code_hash = OwnCodeHash::new();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(address);
            let code_hash_via_own: Hash = own_code_hash.own_code_hash();

            assert_eq!(code_hash_via_own, code_hash);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use ink_e2e::build_message;

        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn get_own_code_hash(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = OwnCodeHashRef::new();
            let contract_acc_id = client
                .instantiate("own_code_hash", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let own_code_hash = build_message::<OwnCodeHashRef>(contract_acc_id)
                .call(|contract| contract.own_code_hash());
            let own_code_hash_res = client
                .call(&ink_e2e::bob(), own_code_hash, 0, None)
                .await
                .expect("own_code_hash failed");

            // Compare codes obtained differently with own_code_hash and code_hash
            let get_code = build_message::<OwnCodeHashRef>(contract_acc_id)
                .call(|contract| contract.get_code());
            let get_code_res = client
                .call(&ink_e2e::alice(), get_code, 0, None)
                .await
                .expect("get_code failed");

            let code_hash_via_own = own_code_hash_res.return_value();
            let code_hash_via_get = get_code_res.return_value();

            assert_eq!(code_hash_via_own, code_hash_via_get);

            Ok(())
        }
    }
}