#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod cross_contract_calls {
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct CrossContractCalls {
        /// Store a contract ref derived from the `address` of an instance
        /// `OtherContract`.
        other_contract: OtherContractRef,
    }

    impl CrossContractCalls {
        /// Initializes the contract with a contract ref
        /// derived from the `address` of an instance `OtherContract`.
        #[ink(constructor)]
        pub fn new(other_contract_address: ink::Address) -> Self {
            let other_contract = OtherContractRef::from(other_contract_address);
            Self { other_contract }
        }

        /// Calls `flip` and `get` on the instance of `OtherContract`
        /// and returns the result of `get`.
        #[ink(message)]
        pub fn flip_and_get(&mut self) -> bool {
            self.other_contract.flip();
            self.other_contract.get()
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod e2e_tests;
