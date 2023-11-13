#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod invoke_contract {
    use ink::env::{
        call::{
            build_call,
            ExecutionInput,
            Selector,
        },
        DefaultEnvironment,
    };

    #[ink(storage)]
    pub struct InvokeContract {}

    impl InvokeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        /// Try to call the `split_profit` function of the contract sent by parameter.
        /// If the account id of the sent contract is not valid it will fail.
        #[ink(message)]
        pub fn invoke_call(&self, contract_to_call: [u8; 32]) -> bool {
            let call = build_call::<DefaultEnvironment>()
                .call(AccountId::from(contract_to_call))
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "im_the_origin"
                ))))
                .returns::<bool>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| {
                    panic!("Received a `LangError`: {:?}", lang_err)
                })
        }
    }

    impl Default for InvokeContract {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use contract_to_call::{
            ContractToCall,
            ContractToCallRef,
        };

        #[ink::test]
        fn call_contract_directly() {
            let contract = ContractToCall::new();
            let is_the_origin = contract.im_the_origin();
            assert!(is_the_origin);
        }

        #[ink::test]
        fn call_contract_indirectly() {
            let contract = InvokeContract::new();
            let code_hash = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                ContractToCallRef,
            >();
            let create_params = ink::env::call::build_create::<ContractToCallRef>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .salt_bytes(&[0_u8; 4])
                .returns::<ContractToCallRef>()
                .params();

            let cr = ink::env
                ::instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from the Contracts pallet while instantiating: {:?}",
                        error
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instatiating: {:?}", error)
                });

            let mut account_id =
                ink::ToAccountId::<DefaultEnvironment>::to_account_id(&cr);
            let account_id: &[u8; 32] = account_id.as_mut();
            let is_the_origin = contract.invoke_call(*account_id);
            assert!(!is_the_origin);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use contract_to_call::{
            ContractToCall,
            ContractToCallRef,
        };
        use ink_e2e::ContractsBackend;

        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "./contract_to_call/Cargo.toml")]
        async fn call_contract_directly(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let mut contract_to_call_constructor = ContractToCallRef::new();

            let contract = client
                .instantiate(
                    "contract_to_call",
                    &ink_e2e::alice(),
                    &mut contract_to_call_constructor,
                )
                .submit()
                .await
                .expect("instantiate failed");
            let call = contract.call::<ContractToCall>();

            // when
            let im_the_origin_call = call.im_the_origin();

            let result = client
                .call(&ink_e2e::alice(), &im_the_origin_call)
                .submit()
                .await;

            // then
            assert!(result
                .expect("This call always returns a value")
                .return_value());

            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "./contract_to_call/Cargo.toml")]
        async fn call_contract_indirectly(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            // given
            let mut original_contract_contructor = InvokeContractRef::new();
            let mut contract_to_call_constructor = ContractToCallRef::new();

            let original_contract = client
                .instantiate(
                    "invoke_contract",
                    &ink_e2e::alice(),
                    &mut original_contract_contructor,
                )
                .submit()
                .await
                .expect("instantiate failed");

            let contract_to_call_acc_id = client
                .instantiate(
                    "contract_to_call",
                    &ink_e2e::alice(),
                    &mut contract_to_call_constructor,
                )
                .submit()
                .await
                .expect("instantiate failed")
                .account_id;

            let call = original_contract.call::<InvokeContract>();

            // when
            let invoke_call = call.invoke_call(*contract_to_call_acc_id.as_ref());

            let result = client.call(&ink_e2e::bob(), &invoke_call).submit().await;

            // then
            assert!(!result
                .expect("This call always returns a value")
                .return_value());

            Ok(())
        }
    }
}
