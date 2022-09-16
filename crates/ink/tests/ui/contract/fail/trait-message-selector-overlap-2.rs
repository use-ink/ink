mod foo1 {
    #[ink::trait_definition(namespace = "same")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

mod foo2 {
    #[ink::trait_definition(namespace = "same")]
    pub trait TraitDefinition {
        #[ink(message)]
        fn message(&self);
    }
}

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
