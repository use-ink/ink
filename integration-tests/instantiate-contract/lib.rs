#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod instantiate_contract {
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct InstantiateContract {}

    impl InstantiateContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn instantiate_other_contract(&self, code_hash: Hash) -> OtherContractRef {
            let create_params = ink::env::call::build_create::<OtherContractRef>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ink::env::call::ExecutionInput::new(
                    ink::env::call::Selector::new(ink::selector_bytes!("new")),
                ))
                .salt_bytes(&[0x0; 4])
                .returns::<OtherContractRef>()
                .params();

            self.env()
                .instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from the Contracts pallet while instantiating: {:?}",
                        error
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instatiating: {:?}", error)
                })
        }
    }

    impl Default for InstantiateContract {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(all(test, feature = "test_instantiate"))]
    mod tests {
        use super::*;

        #[ink::test]
        fn instantiate_other_contract() {
            let contract = InstantiateContract::new();
            let code_hash = Hash::from([0x42; 32]);
            let _ = contract.instantiate_other_contract(code_hash);
        }
    }

    #[cfg(all(test, feature = "e2e-tests", not(feature = "test_instantiate")))]
    mod e2e_tests {
        use ink_e2e::build_message;

        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "other_contract/Cargo.toml")]
        async fn instantiate_other_contract(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let constructor = InstantiateContractRef::new();

            let contract_acc_id = client
                .instantiate(
                    "instantiate_contract",
                    &ink_e2e::bob(),
                    constructor,
                    0,
                    None,
                )
                .await
                .expect("instantiate failed")
                .account_id;

            let other_contract_code_hash = client
                .upload("other_contract", &ink_e2e::bob(), None)
                .await
                .expect("instantiate failed")
                .code_hash;

            let instantiate_other_contract =
                build_message::<InstantiateContractRef>(contract_acc_id.clone())
                    .call(|contract| contract.instantiate_other_contract(other_contract_code_hash));

            let instantiate_other_contract_res = client
                .call_dry_run(&ink_e2e::bob(), &instantiate_other_contract, 0, None)
                .await;

            assert!(instantiate_other_contract_res.exec_result.result.is_ok());

            Ok(())
        }
    }
}
