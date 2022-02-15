use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition1 {
    #[ink(message)]
    fn message(&self);
}

#[ink::trait_definition]
pub trait TraitDefinition2 {
    #[ink(message)]
    fn message(&self);
}

#[ink::contract]
mod contract {
    use super::{
        TraitDefinition1,
        TraitDefinition2,
    };

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }

    impl TraitDefinition1 for Contract {
        #[ink(message)]
        fn message(&self) {}
    }

    impl TraitDefinition2 for Contract {
        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
