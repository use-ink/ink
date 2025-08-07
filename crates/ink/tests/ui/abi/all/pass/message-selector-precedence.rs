#![allow(unexpected_cfgs)]

use contract::Contract;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, name = "myMessage", selector = 1)]
        pub fn message(&self) {}
    }
}

fn main() {
    // For ink! ABI, custom selector (i.e `selector = 1`) takes precedence over name override
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<1_u32>>::SELECTOR,
        1_u32.to_be_bytes(),
    );

    // For Solidity ABI, custom selector (i.e `selector = 1`) is ignored
    // `keccak256("myMessage()")` == `0x1b008a9f`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x1b008a9f_u32>>::SELECTOR,
        [0x1b, 0x00, 0x8a, 0x9f],
    );
}
