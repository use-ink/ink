mod foo1 {
    use ink_lang as ink;

    #[ink::trait_definition(namespace = "same")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

mod foo2 {
    use ink_lang as ink;

    #[ink::trait_definition(namespace = "same")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

use ink_lang as ink;

#[ink::contract]
pub mod contract {
    use super::{
        foo1::TraitDefinition as TraitDefinition1,
        foo2::TraitDefinition as TraitDefinition2,
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
