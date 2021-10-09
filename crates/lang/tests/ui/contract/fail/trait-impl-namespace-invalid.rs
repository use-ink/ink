use ink_lang as ink;

#[ink::trait_definition]
pub trait TraitDefinition {
    #[ink(message)]
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

    #[ink(namespace = "namespace")]
    impl TraitDefinition for Contract {
        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
