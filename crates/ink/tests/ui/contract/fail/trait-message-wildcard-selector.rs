mod foo {
    #[ink::trait_definition]
    pub trait TraitDefinition {
        #[ink(message, selector = _)]
        fn message(&self);
    }
}

#[ink::contract]
pub mod contract {
    use super::foo::TraitDefinition;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }
    }

    impl TraitDefinition for Contract {
        #[ink(message)]
        fn message(&self) {}
    }
}

fn main() {}
