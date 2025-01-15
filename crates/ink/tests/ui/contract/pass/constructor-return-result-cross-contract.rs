#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract_callee {
    #[ink(storage)]
    pub struct Callee {}

    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        Foo,
    }

    impl Callee {
        #[ink(constructor)]
        pub fn new_self() -> Self {
            Self {}
        }

        #[ink(constructor)]
        pub fn new_storage_name() -> Callee {
            Callee {}
        }

        #[ink(constructor)]
        pub fn new_result_self() -> Result<Self, Error> {
            Ok(Self {})
        }

        #[ink(constructor)]
        pub fn new_result_storage_name() -> Result<Callee, Error> {
            Ok(Callee {})
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {
    use contract_callee::{
        CalleeRef,
        Error,
    };

    // fn new_self() -> Self
    let _: fn() -> CalleeRef = || {
        CalleeRef::new_self()
            .code_hash(ink::primitives::H256::zero())
            .endowment(ink::U256::from(25))
            .salt_bytes(Some([1u8; 32]))
            .instantiate()
    };

    // fn new_storage_name() -> Callee
    let _: fn() -> CalleeRef = || {
        CalleeRef::new_storage_name()
            .code_hash(ink::primitives::H256::zero())
            .endowment(ink::U256::from(25))
            .salt_bytes(Some([1u8; 32]))
            .instantiate()
    };

    // fn new_result_self() -> Result<Self, Error>
    let _: fn() -> Result<CalleeRef, Error> = || {
        CalleeRef::new_result_self()
            .code_hash(ink::primitives::H256::zero())
            .endowment(ink::U256::from(25))
            .salt_bytes(Some([1u8; 32]))
            .instantiate()
    };

    // fn new_result_storage_name() -> Result<Callee, Error>
    let _: fn() -> Result<CalleeRef, Error> = || {
        CalleeRef::new_result_self()
            .code_hash(ink::primitives::H256::zero())
            .endowment(ink::U256::from(25))
            .salt_bytes(Some([1u8; 32]))
            .instantiate()
    };
}
