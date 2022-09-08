mod foo1 {
    #[ink::trait_definition]
    pub trait TraitDefinition1 {
        #[ink(message, selector = 42)]
        fn message1(&self);
    }
}

mod foo2 {
    #[ink::trait_definition]
    pub trait TraitDefinition2 {
        #[ink(message, selector = 42)]
        fn message2(&self);
    }
}

#[ink::contract]
pub mod contract {
    use super::{
        foo1::TraitDefinition1,
        foo2::TraitDefinition2,
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
        fn message1(&self) {}
    }

    impl TraitDefinition2 for Contract {
        #[ink(message)]
        fn message2(&self) {}
    }
}

fn main() {}
