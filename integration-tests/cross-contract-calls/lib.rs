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
        /// Initializes the contract by instantiating the code at the given code hash and
        /// storing the resulting account id.
        #[ink(constructor)]
        pub fn new(other_contract_code_hash: Hash) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0)
                .salt_bytes([0xDE, 0xAD, 0xBE, 0xEF])
                .instantiate();

            Self { other_contract }
        }

        /// Basic invocation of the other contract via the contract reference.
        ///
        /// *Note* this will invoke the original `call` (V1) host function.
        #[ink(message)]
        pub fn flip_and_get(&mut self) -> bool {
            self.other_contract.flip();
            self.other_contract.get()
        }

        /// Use the new `call_v2` host function via the call builder to forward calls to
        /// the other contract, initially calling `flip` and then `get` to return the
        /// result.
        ///
        /// This demonstrates how to set the new weight and storage limit parameters via
        /// the call builder API.
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
                .call_v2()
                .ref_time_limit(ref_time_limit)
                .proof_time_limit(proof_time_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke();

            call_builder
                .get()
                .call_v2()
                .ref_time_limit(ref_time_limit)
                .proof_time_limit(proof_time_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke()
        }

        /// Demonstrate that the `call_v2` succeeds without having specified the weight
        /// and storage limit parameters
        #[ink(message)]
        pub fn flip_and_get_invoke_v2_no_weight_limit(&mut self) -> bool {
            let call_builder = self.other_contract.call_mut();

            call_builder.flip().call_v2().invoke();
            call_builder.get().call_v2().invoke()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
