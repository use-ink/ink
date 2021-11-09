mod foo {
    use ink_lang as ink;

    #[ink::trait_definition]
    pub trait TraitDefinition {
        #[ink(message, selector = "_")]
        fn message1(&self);
    }
}

use ink_lang as ink;

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
        fn message1(&self) {}
    }
}

fn main() {}
