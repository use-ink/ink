#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod basic_contract_caller {
    /// We import the generated `ContractRef` of our other contract.
    ///
    /// Note that the other contract must have re-exported it (`pub use
    /// OtherContractRef`) for us to have access to it.
    use other_contract::OtherContractRef;

    #[ink(storage)]
    pub struct BasicContractCaller {
        /// We specify that our contract will store a reference to the `OtherContract`.
        other_contract: OtherContractRef,
    }

    impl BasicContractCaller {
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

            Self { other_contract }
        }

        /// Using the `ContractRef` we can call all the messages of the `OtherContract` as
        /// if they were normal Rust methods (because at the end of the day, they
        /// are!).
        #[ink(message)]
        pub fn flip_and_get(&mut self) -> bool {
            self.other_contract.flip();
            self.other_contract.get()
        }
    }
}
