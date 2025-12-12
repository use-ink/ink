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
                    Abi::Ink,
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
                    Abi::Ink,
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
                    Abi::Ink,
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
                    Abi::Ink,
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
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        Abi::Ink,
                        "set_x"
                    )))
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
                    ExecutionInput::new(Selector::new(ink::selector_bytes!(
                        Abi::Ink,
                        "set_x"
                    )))
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
mod tests;

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;

}
