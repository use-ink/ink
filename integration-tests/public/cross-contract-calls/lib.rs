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
        /// Initializes the contract by instantiating the code at the given code hash via
        /// `instantiate` host function with the supplied weight and storage
        /// limits.
        #[ink(constructor)]
        pub fn new_with_limits(
            other_contract_code_hash: ink::H256,
            ref_time_limit: u64,
            proof_size_limit: u64,
            storage_deposit_limit: ink::U256,
        ) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0.into())
                .salt_bytes(Some([1u8; 32]))
                .ref_time_limit(ref_time_limit)
                .proof_size_limit(proof_size_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .instantiate();

            Self { other_contract }
        }

        /// Initializes the contract by instantiating the code at the given code hash via
        /// the `instantiate` host function with no weight or storage limits.
        #[ink(constructor)]
        pub fn new_no_limits(other_contract_code_hash: ink::H256) -> Self {
            let other_contract = OtherContractRef::new(true)
                .code_hash(other_contract_code_hash)
                .endowment(0.into())
                .salt_bytes(Some([1u8; 32]))
                .instantiate();

            Self { other_contract }
        }

        /// Use the `call` host function via the call builder to forward calls to
        /// the other contract, initially calling `flip` and then `get` to return the
        /// result.
        ///
        /// This demonstrates how to set the new weight and storage limit parameters via
        /// the call builder API.
        #[ink(message)]
        pub fn flip_and_get_invoke_with_limits(
            &mut self,
            ref_time_limit: u64,
            proof_size_limit: u64,
            storage_deposit_limit: ink::U256,
        ) -> bool {
            let call_builder = self.other_contract.call_mut();

            call_builder
                .flip()
                .ref_time_limit(ref_time_limit)
                .proof_size_limit(proof_size_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke();

            call_builder
                .get()
                .ref_time_limit(ref_time_limit)
                .proof_size_limit(proof_size_limit)
                .storage_deposit_limit(storage_deposit_limit)
                .invoke()
        }

        /// Demonstrate that the `call` succeeds without having specified the weight
        /// and storage limit parameters
        #[ink(message)]
        pub fn flip_and_get_invoke_no_weight_limit(&mut self) -> bool {
            self.other_contract.flip();
            self.other_contract.get()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
