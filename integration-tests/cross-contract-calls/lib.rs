#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod cross_contract_calls {
    use ink::{
        codegen::ContractCallBuilder,
        env::{
            ContractEnv,
            call::FromAccountId
        },
    };
    use other_contract::{OtherContract, OtherContractRef};

    type Env = <CrossContractCalls as ContractEnv>::Env;
    type OtherContractCallBuilder = <OtherContract as ContractCallBuilder>::Type;

    #[ink(storage)]
    pub struct CrossContractCalls {
        other_contract_call_builder: OtherContractCallBuilder,
    }

    impl CrossContractCalls {
        /// In order to use the `OtherContract` we first need to **instantiate** it.
        ///
        /// To do this we will use the uploaded `code_hash` of `OtherContract`.
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            let other_contract_call_builder = <OtherContractCallBuilder as FromAccountId<Env>>::from_account_id(
                *other_contract.as_ref(),
            );

            Self { other_contract_call_builder }
        }

        /// Using the `ContractRef` we can call all the messages of the `OtherContract` as
        /// if they were normal Rust methods (because at the end of the day, they
        /// are!).
        #[ink(message)]
        pub fn flip_and_get(&mut self) -> bool {
            self.other_contract_call_builder
                .flip()
                // .v2()
                // .weight_limit(0, 0)
                .invoke();

            self.other_contract_call_builder
                .get()
                // .v2()
                .invoke()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
