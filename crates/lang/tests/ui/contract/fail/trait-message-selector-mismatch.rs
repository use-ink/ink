use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message, selector = 1)]
    fn message(&self);
}

#[ink::contract]
mod contract {
    use super::TraitDefinition;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }

    impl TraitDefinition for Contract {
        #[ink(message, selector = 2)]
        fn message(&self) {}
    }
}

fn main() {}
