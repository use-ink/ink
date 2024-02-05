#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod cross_contract_calls {
    use ink::codegen::TraitCallBuilder;
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct CrossContractCalls {
        other_contract: OtherContractRef,
    }

    impl CrossContractCalls {
        /// todo: [AJ] comment
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            Self { other_contract }
        }

        /// todo: [AJ] comment
        #[ink(message)]
        pub fn flip_and_get_invoke_v2_with_limits(
            &mut self,
            ref_time_limit: u64,
            proof_time_limit: u64,
            storage_deposit_limit: Balance,
        ) -> bool {
            let call_builder = self.other_contract.call_mut();

            call_builder
                .flip()
                .v2()
                .ref_time_limit(ref_time_limit)
                .proof_time_limit(proof_time_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke();

            call_builder
                .get()
                .v2()
                .ref_time_limit(ref_time_limit)
                .proof_time_limit(proof_time_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke()
        }

        #[ink(message)]
        pub fn flip_and_get_invoke_v2_no_weight_limit(&mut self) -> bool {
            let call_builder = self.other_contract.call_mut();

            call_builder.flip().v2().invoke();
            call_builder.get().v2().invoke()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
