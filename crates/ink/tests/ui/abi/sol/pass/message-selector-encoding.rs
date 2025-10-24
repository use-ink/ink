#![allow(unexpected_cfgs)]

use contract::Contract;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink::trait_definition]
    pub trait Messages {
        #[ink(message)]
        fn message_0(&self);

        #[ink(message)]
        fn message_1(&self);
    }

    impl Messages for Contract {
        #[ink(message)]
        fn message_0(&self) {}

        #[ink(message)]
        fn message_1(&self) {}
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_2(&self) {}

        #[ink(message)]
        pub fn message_3(&self) {}
    }

    #[ink::trait_definition]
    pub trait Messages2 {
        #[ink(message)]
        fn message_4(&self);
    }

    impl Messages2 for Contract {
        #[ink(message)]
        fn message_4(&self) {}
    }
}

fn main() {
    // `keccak256("message_0()")` == `0xdf4bc0a6`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xdf4bc0a6_u32>>::SELECTOR,
        [0xdf, 0x4b, 0xc0, 0xa6],
    );

    // `keccak256("message_1()")` == `0x20896153`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x20896153_u32>>::SELECTOR,
        [0x20, 0x89, 0x61, 0x53],
    );

    // `keccak256("message_2()")` == `0x551578a6`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x551578a6_u32>>::SELECTOR,
        [0x55, 0x15, 0x78, 0xa6],
    );

    // `keccak256("message_3()")` == `0xb2f14ed9`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xb2f14ed9_u32>>::SELECTOR,
        [0xb2, 0xf1, 0x4e, 0xd9],
    );

    // `keccak256("message_4()")` == `0xdc48aa5a`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xdc48aa5a_u32>>::SELECTOR,
        [0xdc, 0x48, 0xaa, 0x5a],
    );
}
