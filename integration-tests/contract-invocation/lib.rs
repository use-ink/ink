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
        pub fn instantiate_contract1(&self, code_hash: Hash, salt: u32) -> Contract1Ref {
            let salt = salt.to_le_bytes();
            let create_params = build_create::<Contract1Ref>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ExecutionInput::new(
                    Selector::new(ink::selector_bytes!("new")),
                ))
                .salt_bytes(&salt)
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
        pub fn instantiate_contract2(&self, code_hash: Hash, salt: u32) -> Contract2Ref {
            let salt = salt.to_le_bytes();
            let create_params = build_create::<Contract2Ref>()
                .code_hash(code_hash)
                .gas_limit(0)
                .endowment(0)
                .exec_input(ExecutionInput::new(
                    Selector::new(ink::selector_bytes!("new")),
                ))
                .salt_bytes(&salt)
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
        use virtual_contract::VirtualContractRef;
        use virtual_contract_ver1::VirtualContractVer1Ref;
        use virtual_contract_ver2::VirtualContractVer2Ref;

        fn instantiate_contract1(contract: &ContractTester, code_hash: Hash, salt: u32) -> AccountId{
            let cr = contract.instantiate_contract1(code_hash, salt);
            ink::ToAccountId::<DefaultEnvironment>::to_account_id(&cr)
        }

        fn instantiate_contract2(contract: &ContractTester, code_hash: Hash, salt: u32) -> AccountId{
            let cr = contract.instantiate_contract2(code_hash, salt);
            ink::ToAccountId::<DefaultEnvironment>::to_account_id(&cr)
        }

        fn to_array(address: &mut AccountId) -> [u8; 32]{
            let temp: &[u8; 32] = address.as_mut();
            *temp
        }

        #[ink::test]
        fn test_invoke() {
            let contract = ContractTester::new();
            let code_hash1 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, Contract1Ref>();
            let code_hash2 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, Contract2Ref>();

            let mut contract1_address1_account = instantiate_contract1(&contract, code_hash1, 1);
            let mut contract1_address2_account = instantiate_contract1(&contract, code_hash1, 2);
            let mut contract2_address1_account = instantiate_contract2(&contract, code_hash2, 3);
            let mut contract2_address2_account = instantiate_contract2(&contract, code_hash2, 4);

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
            check_hashes(&contract1_address1_account, &contract1_address2_account, code_hash1);
            check_hashes(&contract2_address1_account, &contract2_address2_account, code_hash2);

            let check_values1 = |a, b| {
                let x = contract.contract1_get_x(contract1_address1);
                let y = contract.contract1_get_x(contract1_address2);
                assert_eq!(x, a);
                assert_eq!(y, b);
            };
            let check_values2 = |a, b| {
                let x = contract.contract2_get_x(contract2_address1);
                let y = contract.contract2_get_x(contract2_address2);
                assert_eq!(x, a);
                assert_eq!(y, b);
            };

            check_values1(42, 42);
            check_values2(123456, 123456);
            contract.contract2_set_x(contract2_address1, 78);
            check_values1(42, 42);
            check_values2(123456, 123456);
            contract.contract1_set_x(contract1_address1, 123);
            contract.contract2_set_x(contract2_address2, 87);
            check_values1(123, 42);
            check_values2(123456, 123456);
            contract.contract1_set_x(contract1_address2, 321);
            check_values1(123, 321);
            check_values2(123456, 123456);

        }

        #[ink::test]
        fn test_invoke_delegate() {
            let code_hash1 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, VirtualContractRef>();
            let code_hash2 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, VirtualContractVer1Ref>();
            let code_hash3 = ink::env::test::upload_code::<ink::env::DefaultEnvironment, VirtualContractVer2Ref>();

            let ic = |hash, salt: u32, x|{
                let salt = salt.to_le_bytes();
                let create_params = build_create::<VirtualContractRef>()
                    .code_hash(code_hash1)
                    .gas_limit(0)
                    .endowment(0)
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!("new")))
                            .push_arg(hash)
                            .push_arg(x),
                    )
                    .salt_bytes(&salt)
                    .returns::<VirtualContractRef>()
                    .params();

                ink::env::instantiate_contract(&create_params)
                    .unwrap_or_else(|error| {
                        panic!(
                            "Received an error from the Contracts pallet while instantiating: {:?}",
                            error
                        )
                    })
                    .unwrap_or_else(|error| {
                        panic!("Received a `LangError` while instatiating: {:?}", error)
                    })
            };

            let mut ref1 = ic(code_hash2, 1, 42);
            let mut ref2 = ic(code_hash3, 2, 74);

            let check_values = |r1: &VirtualContractRef, r2: &VirtualContractRef, a, b, c, d|{
                let v1 = r1.real_get_x();
                let v2 = r2.real_get_x();
                let v3 = r1.get_x();
                let v4 = r2.get_x();
                assert_eq!(v1, a);
                assert_eq!(v2, b);
                assert_eq!(v3, c);
                assert_eq!(v4, d);
            };

            check_values(&ref1, &ref2, 42, 74, 43, 148);
            ref1.set_x(15);
            check_values(&ref1, &ref2, 42, 74, 43, 148);
            ref1.real_set_x(15);
            check_values(&ref1, &ref2, 15, 74, 16, 148);
            ref2.set_x(39);
            check_values(&ref1, &ref2, 15, 74, 16, 148);
            ref2.real_set_x(39);
            check_values(&ref1, &ref2, 15, 39, 16, 78);

        }
    }
}
