#![cfg_attr(not(feature = "std"), no_main)]

pub type TyAlias1 = i32;
pub type TyAlias2 = TyAlias1;

#[ink::contract]
pub mod primitive_topic {

    #[ink(event, anonymous)]
    pub struct Transaction {
        // Bad
        #[ink(topic)]
        value_1: u8,
        // Bad
        #[ink(topic)]
        value_2: Balance,
        // Bad
        #[ink(topic)]
        value_3: crate::TyAlias1,
        // Bad
        #[ink(topic)]
        value_4: crate::TyAlias2,
    }

    #[ink(storage)]
    pub struct PrimitiveTopic {}

    impl PrimitiveTopic {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }
        #[ink(message)]
        pub fn do_nothing(&mut self) {}
    }
}

fn main() {}
