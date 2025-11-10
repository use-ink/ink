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

        #[ink(message, selector = 1)]
        fn message_1(&self);
    }

    impl Messages for Contract {
        #[ink(message)]
        fn message_0(&self) {}

        #[ink(message, selector = 1)]
        fn message_1(&self) {}
    }

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = 0xC0DE_CAFE)]
        pub fn message_2(&self) {}

        #[ink(message)]
        pub fn message_3(&self) {}
    }

    #[ink::trait_definition]
    pub trait Messages2 {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self);
    }

    impl Messages2 for Contract {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self) {}
    }
}

fn main() {
    // ink! `message_0` trait definition
    const TRAIT_ID: u32 = ::ink::selector_id!("Messages::message_0");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<TRAIT_ID>>::SELECTOR,
        [0xFB, 0xAB, 0x03, 0xCE],
    );
    // Solidity `keccak256("message_0()")` == `0xdf4bc0a6`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xdf4bc0a6_u32>>::SELECTOR,
        [0xdf, 0x4b, 0xc0, 0xa6],
    );

    // ink! `message_1` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<1_u32>>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    // Solidity `keccak256("message_1()")` == `0x20896153`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x20896153_u32>>::SELECTOR,
        [0x20, 0x89, 0x61, 0x53],
    );

    // ink! `message_2` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xC0DE_CAFE_u32>>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
    // Solidity `keccak256("message_2()")` == `0x551578a6`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x551578a6_u32>>::SELECTOR,
        [0x55, 0x15, 0x78, 0xa6],
    );

    // ink! `message_3` inherent
    const INHERENT_ID: u32 = ::ink::selector_id!("message_3");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<INHERENT_ID>>::SELECTOR,
        [0xB6, 0xC3, 0x27, 0x49],
    );
    // Solidity `keccak256("message_3()")` == `0xb2f14ed9`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xb2f14ed9_u32>>::SELECTOR,
        [0xb2, 0xf1, 0x4e, 0xd9],
    );

    // ink! `message_4` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x12345678_u32>>::SELECTOR,
        0x12345678_u32.to_be_bytes(),
    );
    // Solidity `keccak256("message_4()")` == `0xdc48aa5a`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xdc48aa5a_u32>>::SELECTOR,
        [0xdc, 0x48, 0xaa, 0x5a],
    );
}
