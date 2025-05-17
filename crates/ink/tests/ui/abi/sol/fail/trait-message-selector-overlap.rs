#![allow(unexpected_cfgs)]

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
    fn message(&self);
}

#[ink::contract]
pub mod contract {
    use super::TraitDefinition;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }

    impl TraitDefinition for Contract {
        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
