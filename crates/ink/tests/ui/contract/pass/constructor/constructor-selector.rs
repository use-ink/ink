#![allow(unexpected_cfgs)]

use contract::Contract;
use ink::selector_bytes;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor_0() -> Self {
            Self {}
        }

        #[ink(constructor, selector = 1)]
        pub fn constructor_1() -> Self {
            Self {}
        }

        #[ink(constructor, selector = 0xC0DE_CAFE)]
        pub fn constructor_2() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {
    const ID: u32 = ::ink::selector_id!("constructor_0");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<ID>>::SELECTOR.unwrap(),
        selector_bytes!("constructor_0")
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<1_u32>>::SELECTOR
            .unwrap(),
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<0xC0DE_CAFE_u32>>::SELECTOR.unwrap(),
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
}
