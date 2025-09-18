#![allow(unexpected_cfgs)]

use contract::Contract;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor, name = "myConstructor", selector = 1)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {
    // Custom selector (i.e `selector = 1`) takes precedence over name override
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<1_u32>>::SELECTOR.unwrap(),
        1_u32.to_be_bytes(),
    );
}
