#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod instantiate_contract {
    use contract1::Contract1Ref;
    use contract2::Contract2Ref;
    use ink::env::{
        call::{
            build_create,
            build_call,
            ExecutionInput,
            Selector,
        },
    };

    #[ink(storage)]
    pub struct ContractTester {}

    impl ContractTester {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self{ }
        }

        #[ink(message)]
        pub fn instantiate_contract1(&self, code_hash: Hash) -> Contract1Ref {
            let create_params = build_create::<Contract1Ref>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ExecutionInput::new(
                    Selector::new(ink::selector_bytes!("new")),
                ))
                .salt_bytes(&[0x0; 4])
                .returns::<Contract1Ref>()
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

        #[ink(message)]
        pub fn instantiate_contract2(&self, code_hash: Hash) -> Contract2Ref {
            let create_params = build_create::<Contract2Ref>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ExecutionInput::new(
                    Selector::new(ink::selector_bytes!("new")),
                ))
                .salt_bytes(&[0x0; 4])
                .returns::<Contract2Ref>()
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

        #[ink(message)]
        pub fn contract1_get_x(&self, contract1_address: [u8; 32]) -> u32 {
            let call = build_call()
                .call(AccountId::from(contract1_address))
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_x")))
                )
                .returns::<u32>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
        }

        #[ink(message)]
        pub fn contract2_get_x(&self, contract2_address: [u8; 32]) -> u32 {
            let call = build_call()
                .call(AccountId::from(contract2_address))
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("get_x")))
                )
                .returns::<u32>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
        }

        #[ink(message)]
        pub fn contract1_set_x(&self, contract1_address: [u8; 32], new_x: u32) {
            let call = ink::env::call::build_call()
                .call(AccountId::from(contract1_address))
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_x")))
                        .push_arg(new_x)
                )
                .returns::<()>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
        }

        #[ink(message)]
        pub fn contract2_set_x(&self, contract2_address: [u8; 32], new_x: u64) {
            let call = ink::env::call::build_call()
                .call(AccountId::from(contract2_address))
                .gas_limit(0)
                .transferred_value(0)
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_x")))
                        .push_arg(new_x)
                )
                .returns::<()>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {:?}", lang_err))
        }
    }

    impl Default for ContractTester {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(all(test, feature = "test_instantiate"))]
    mod tests {
        use super::*;
        use ink::{
            env::{
                DefaultEnvironment,
            },
            primitives::AccountId,
        };

        fn instantiate_contract1(contract: &ContractTester, code_hash: Hash) -> AccountId{
            let cr = contract.instantiate_contract1(code_hash);
            ink::ToAccountId::<DefaultEnvironment>::to_account_id(&cr)
        }

        fn instantiate_contract2(contract: &ContractTester, code_hash: Hash) -> AccountId{
            let cr = contract.instantiate_contract2(code_hash);
            ink::ToAccountId::<DefaultEnvironment>::to_account_id(&cr)
        }

        fn to_array(address: &mut AccountId) -> [u8; 32]{
            let temp: &[u8; 32] = address.as_mut();
            temp.clone()
        }

        #[ink::test]
        fn test_contract1() {
            let contract = ContractTester::new();
            let code_hash1 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, Contract1Ref>();
            let code_hash2 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, Contract2Ref>();

            // Set different caller AccountIds, so that instantiate_contract will generate different account ids.
            ink::env::test::set_caller::<DefaultEnvironment>([1_u8; 32].into());
            let mut contract1_address1_account = instantiate_contract1(&contract, code_hash1);
            let mut contract2_address1_account = instantiate_contract2(&contract, code_hash2);
            ink::env::test::set_caller::<DefaultEnvironment>([2_u8; 32].into());
            let mut contract1_address2_account = instantiate_contract1(&contract, code_hash1);
            let mut contract2_address2_account = instantiate_contract2(&contract, code_hash2);

            let contract1_address1 = to_array(&mut contract1_address1_account);
            let contract1_address2 = to_array(&mut contract1_address2_account);
            let contract2_address1 = to_array(&mut contract2_address1_account);
            let contract2_address2 = to_array(&mut contract2_address2_account);

            let check_hashes = |a, b, c|{
                let x = ink::env::code_hash::<DefaultEnvironment>(a)
                    .expect("failed to get code hash");
                let y = ink::env::code_hash::<DefaultEnvironment>(b)
                    .expect("failed to get code hash");

                assert_eq!(x, c);
                assert_eq!(y, c);
            };
            check_hashes(&contract1_address1_account, &contract1_address2_account, code_hash1.clone());
            check_hashes(&contract2_address1_account, &contract2_address2_account, code_hash2.clone());

            let check_values1 = |a, b| {
                let x = contract.contract1_get_x(contract1_address1.clone());
                let y = contract.contract1_get_x(contract1_address2.clone());
                assert_eq!(x, a);
                assert_eq!(y, b);
            };
            let check_values2 = |a, b| {
                let x = contract.contract2_get_x(contract2_address1.clone());
                let y = contract.contract2_get_x(contract2_address2.clone());
                assert_eq!(x, a);
                assert_eq!(y, b);
            };

            check_values1(42, 42);
            check_values2(123456, 123456);
            contract.contract2_set_x(contract2_address1.clone(), 78);
            check_values1(42, 42);
            check_values2(123456, 123456);
            contract.contract1_set_x(contract1_address1.clone(), 123);
            contract.contract2_set_x(contract2_address2.clone(), 87);
            check_values1(123, 42);
            check_values2(123456, 123456);
            contract.contract1_set_x(contract1_address2.clone(), 321);
            check_values1(123, 321);
            check_values2(123456, 123456);

        }
    }

    #[cfg(all(test, feature = "e2e-tests", not(feature = "test_instantiate")))]
    mod e2e_tests {
        use ink_e2e::build_message;

        use super::*;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "other_contract/Cargo.toml")]
        async fn instantiate_other_contract(
            mut client: ink_e2e::Client<C, E>,
        ) -> E2EResult<()> {
            let constructor = ContractTesterRef::new();

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

            let instantiate_other_contract = build_message::<ContractTesterRef>(
                contract_acc_id.clone(),
            )
            .call(|contract| {
                contract.instantiate_other_contract(other_contract_code_hash)
            });

            let instantiate_other_contract_res = client
                .call_dry_run(&ink_e2e::bob(), &instantiate_other_contract, 0, None)
                .await;

            assert!(instantiate_other_contract_res.exec_result.result.is_ok());

            Ok(())
        }
    }
}
