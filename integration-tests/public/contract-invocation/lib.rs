#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod instantiate_contract {
    use contract1::Contract1Ref;
    use contract2::Contract2Ref;
    use ink::{
        H256,
        env::call::{
            ExecutionInput,
            Selector,
            build_call,
            build_create,
        },
    };

    #[ink(storage)]
    pub struct ContractTester {}

    impl ContractTester {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn instantiate_contract1(&self, code_hash: H256, salt: u32) -> Contract1Ref {
            let mut salt_bytes = [0u8; 32];
            salt_bytes[..4].copy_from_slice(&salt.to_le_bytes());

            let create_params = build_create::<Contract1Ref>()
                .code_hash(code_hash)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .salt_bytes(Some(salt_bytes))
                .returns::<Contract1Ref>()
                .params();

            self.env()
                .instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                })
        }

        #[ink(message)]
        pub fn instantiate_contract2(&self, code_hash: H256, salt: u32) -> Contract2Ref {
            let mut salt_bytes = [0u8; 32];
            salt_bytes[..4].copy_from_slice(&salt.to_le_bytes());
            let create_params = build_create::<Contract2Ref>()
                .code_hash(code_hash)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .salt_bytes(Some(salt_bytes))
                .returns::<Contract2Ref>()
                .params();

            self.env()
                .instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                })
        }

        #[ink(message)]
        pub fn contract1_get_x(&self, contract1_address: Address) -> u32 {
            let call = build_call()
                .call(contract1_address)
                .transferred_value(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get_x"
                ))))
                .returns::<u32>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }

        #[ink(message)]
        pub fn contract2_get_x(&self, contract2_address: Address) -> u32 {
            let call = build_call()
                .call(contract2_address)
                .transferred_value(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "get_x"
                ))))
                .returns::<u32>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }

        #[ink(message)]
        pub fn contract1_set_x(&self, contract1_address: Address, new_x: u32) {
            let call = ink::env::call::build_call()
                .call(contract1_address)
                .transferred_value(0.into())
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_x")))
                        .push_arg(new_x),
                )
                .returns::<()>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }

        #[ink(message)]
        pub fn contract2_set_x(&self, contract2_address: Address, new_x: u64) {
            let call = ink::env::call::build_call()
                .call(contract2_address)
                .transferred_value(0.into())
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("set_x")))
                        .push_arg(new_x),
                )
                .returns::<()>()
                .params();

            self.env()
                .invoke_contract(&call)
                .unwrap_or_else(|env_err| {
                    panic!("Received an error from the Environment: {env_err:?}")
                })
                .unwrap_or_else(|lang_err| panic!("Received a `LangError`: {lang_err:?}"))
        }
    }

    impl Default for ContractTester {
        fn default() -> Self {
            Self::new()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use virtual_contract::VirtualContractRef;
        use virtual_contract_ver1::VirtualContractVer1Ref;
        use virtual_contract_ver2::VirtualContractVer2Ref;

        fn instantiate_contract1(
            contract: &ContractTester,
            code_hash: H256,
            salt: u32,
        ) -> Address {
            let cr = contract.instantiate_contract1(code_hash, salt);
            ink::ToAddr::to_addr(&cr)
        }

        fn instantiate_contract2(
            contract: &ContractTester,
            code_hash: H256,
            salt: u32,
        ) -> Address {
            let cr = contract.instantiate_contract2(code_hash, salt);
            ink::ToAddr::to_addr(&cr)
        }

        #[ink::test]
        fn test_invoke() {
            let contract = ContractTester::new();
            let code_hash1 = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                Contract1Ref,
            >();
            let code_hash2 = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                Contract2Ref,
            >();

            let contract1_address1 = instantiate_contract1(&contract, code_hash1, 1);
            let contract1_address2 = instantiate_contract1(&contract, code_hash1, 2);
            let contract2_address1 = instantiate_contract2(&contract, code_hash2, 3);
            let contract2_address2 = instantiate_contract2(&contract, code_hash2, 4);

            let check_hashes = |a, b, c| {
                let x = ink::env::code_hash(a).expect("failed to get code hash");
                let y = ink::env::code_hash(b).expect("failed to get code hash");

                assert_eq!(x, c);
                assert_eq!(y, c);
            };
            check_hashes(&contract1_address1, &contract1_address2, code_hash1);
            check_hashes(&contract2_address1, &contract2_address2, code_hash2);

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
            let code_hash1 = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                VirtualContractRef,
            >();
            let code_hash2 = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                VirtualContractVer1Ref,
            >();
            let code_hash3 = ink::env::test::upload_code::<
                ink::env::DefaultEnvironment,
                VirtualContractVer2Ref,
            >();

            let create_params = build_create::<VirtualContractVer1Ref>()
                .code_hash(code_hash2)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .returns::<VirtualContractVer1Ref>()
                .params();

            let addr2 = ink::env::instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                });

            use ink::ToAddr;
            let addr2: Address = addr2.to_addr();

            let create_params = build_create::<VirtualContractVer2Ref>()
                .code_hash(code_hash3)
                .endowment(0.into())
                .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
                    "new"
                ))))
                .returns::<VirtualContractVer2Ref>()
                .params();

            let addr3 = ink::env::instantiate_contract(&create_params)
                .unwrap_or_else(|error| {
                    panic!(
                        "Received an error from `pallet-revive` while instantiating: {error:?}"
                    )
                })
                .unwrap_or_else(|error| {
                    panic!("Received a `LangError` while instantiating: {error:?}")
                });
            let addr3: Address = addr3.to_addr();

            // creates `code_hash1` contract and puts `hash` + `x` as the constructor
            // arguments
            let instantiate = |delegate_addr: Address, salt: u32, x| {
                let mut salt_bytes = [0u8; 32];
                salt_bytes[..4].copy_from_slice(&salt.to_le_bytes());
                let create_params = build_create::<VirtualContractRef>()
                    .code_hash(code_hash1)
                    .endowment(0.into())
                    .exec_input(
                        ExecutionInput::new(Selector::new(ink::selector_bytes!("new")))
                            //.push_arg(H256::zero()) // todo should result in err, but doesn't
                            .push_arg(delegate_addr)
                            .push_arg(x),
                    )
                    .salt_bytes(Some(salt_bytes))
                    .returns::<VirtualContractRef>()
                    .params();

                ink::env::instantiate_contract(&create_params)
                    .unwrap_or_else(|error| {
                        panic!(
                            "Received an error from `pallet-revive` while instantiating: {error:?}"
                        )
                    })
                    .unwrap_or_else(|error| {
                        panic!("Received a `LangError` while instantiating: {error:?}")
                    })
            };

            let mut ref1 = instantiate(addr2, 1, 42);
            let mut ref2 = instantiate(addr3, 2, 74);

            let check_values =
                |r1: &VirtualContractRef, r2: &VirtualContractRef, a, b, c, d| {
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

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use ink_e2e::{
            ChainBackend,
            ContractsBackend,
            E2EBackend,
            InstantiationResult,
        };
        use virtual_contract::{
            VirtualContractRef,
            virtual_contract::VirtualContract,
        };
        use virtual_contract_ver1::VirtualContractVer1Ref;
        use virtual_contract_ver2::VirtualContractVer2Ref;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        async fn check_values<Client, E, B>(
            origin: &ink_e2e::Keypair,
            client: &mut Client,

            ver1: &InstantiationResult<E, B, ink::env::DefaultAbi>,
            ver2: &InstantiationResult<E, B, ink::env::DefaultAbi>,

            a: u32,
            b: u32,
            c: u32,
            d: u32,
        ) where
            Client: E2EBackend<ink::env::DefaultEnvironment>,
            E: ink::env::Environment,
        {
            let r1 = ver1.call_builder::<VirtualContract>();
            let r2 = ver2.call_builder::<VirtualContract>();

            let v1_get = r1.real_get_x();
            let v1 = client
                .call(&origin, &v1_get)
                .dry_run()
                .await
                .unwrap_or_else(|_| panic!("foo"))
                .return_value();

            let v2_get = r2.real_get_x();
            let v2 = client
                .call(&origin, &v2_get)
                .dry_run()
                .await
                .unwrap_or_else(|_| panic!("foo"))
                .return_value();

            let v3_get = r1.get_x();
            let v3 = client
                .call(&origin, &v3_get)
                .dry_run()
                .await
                .unwrap_or_else(|_| panic!("foo"))
                .return_value();

            let v4_get = r2.get_x();
            let v4 = client
                .call(&origin, &v4_get)
                .dry_run()
                .await
                .unwrap_or_else(|_| panic!("foo"))
                .return_value();

            assert_eq!(v1, a);
            assert_eq!(v2, b);
            assert_eq!(v3, c);
            assert_eq!(v4, d);
        }

        #[ink_e2e::test]
        async fn test_invoke_delegate_e2e<Client: E2EBackend>(
            mut client: Client,
        ) -> E2EResult<()> {
            let origin = client
                .create_and_fund_account(&ink_e2e::alice(), 10_000_000_000_000)
                .await;

            let mut constructor = VirtualContractVer1Ref::new();
            let addr_virtual_ver1 = client
                .instantiate("virtual_contract_ver1", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate `delegatee` failed");

            let mut constructor = VirtualContractVer2Ref::new();
            let addr_virtual_ver2 = client
                .instantiate("virtual_contract_ver2", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate `delegatee` failed");

            let mut constructor = VirtualContractRef::new(addr_virtual_ver1.addr, 42);
            let ver1 = client
                .instantiate("virtual_contract", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate `delegatee` failed");

            let mut constructor = VirtualContractRef::new(addr_virtual_ver2.addr, 74);
            let ver2 = client
                .instantiate("virtual_contract", &origin, &mut constructor)
                .submit()
                .await
                .expect("instantiate `delegatee` failed");

            // contract code_hash1 with argument code_hash2
            check_values(&origin, &mut client, &ver1, &ver2, 42, 74, 43, 148).await;

            let mut call_builder = ver1.call_builder::<VirtualContract>();
            let call = call_builder.set_x(15);
            let _call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call` failed");

            check_values(&origin, &mut client, &ver1, &ver2, 42, 74, 43, 148).await;

            let mut call_builder = ver1.call_builder::<VirtualContract>();
            let call = call_builder.real_set_x(15);
            let _call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call` failed");

            check_values(&origin, &mut client, &ver1, &ver2, 15, 74, 16, 148).await;

            let mut call_builder = ver2.call_builder::<VirtualContract>();
            let call = call_builder.set_x(39);
            let _call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call` failed");

            check_values(&origin, &mut client, &ver1, &ver2, 15, 74, 16, 148).await;

            let mut call_builder = ver2.call_builder::<VirtualContract>();
            let call = call_builder.real_set_x(39);
            let _call_result = client
                .call(&origin, &call)
                .submit()
                .await
                .expect("Calling `call_builder::call` failed");

            check_values(&origin, &mut client, &ver1, &ver2, 15, 39, 16, 78).await;

            Ok(())
        }
    }
}
