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

        #[ink(message)]
        pub fn message_0(&self, _input_1: bool) {}

        #[ink(message, selector = 1)]
        pub fn message_1(&self, _input_1: u8) {}

        #[ink(message, selector = 0xC0DE_CAFE)]
        pub fn message_2(&self, _input_1: ink::sol::FixedBytes<32>) {}

        #[ink(message)]
        pub fn message_3(&self, _input_1: bool, _input_2: i8) {}
    }

    #[ink::trait_definition]
    pub trait Messages {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self, _input_1: ink::sol::DynBytes);
    }

    impl Messages for Contract {
        #[ink(message, selector = 0x12345678)]
        fn message_4(&self, _input_1: ink::sol::DynBytes) {}
    }

    impl Contract {
        #[ink(message)]
        pub fn message_5(&self, _input_1: String) {}

        #[ink(message)]
        pub fn message_6(
            &self,
            _input_1: bool,
            _input_2: String,
            _input_3: ink::sol::FixedBytes<32>,
            _input_4: ink::sol::DynBytes,
            _input_5: [u8; 4],
            _input_6: Vec<u8>,
        ) {
        }

        #[ink(message)]
        pub fn message_7(&self, _input_1: AccountId, _input_2: Hash, _input_3: Address) {}
    }
}

fn main() {
    // ink! inherent `blake2bx256(message_0)` == `0x5a6ac15d`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x5a6ac15d_u32>>::SELECTOR,
        [0x5a, 0x6a, 0xc1, 0x5d],
    );
    // `keccak256("message_0(bool)")` == `0xc58b7175`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xc58b7175_u32>>::SELECTOR,
        [0xc5, 0x8b, 0x71, 0x75],
    );

    // ink! `message_1` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<1_u32>>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    // `keccak256("message_1(uint8)")` == `0xe4634c56`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xe4634c56_u32>>::SELECTOR,
        [0xe4, 0x63, 0x4c, 0x56],
    );

    // ink! `message_2` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xC0DE_CAFE_u32>>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
    // `keccak256("message_2(bytes32)")` == `0x468f916c`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x468f916c_u32>>::SELECTOR,
        [0x46, 0x8f, 0x91, 0x6c],
    );

    // ink! inherent `blake2bx256(message_3)` == `0xB6C32749`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xB6C32749_u32>>::SELECTOR,
        [0xB6, 0xC3, 0x27, 0x49],
    );
    // `keccak256("message_3(bool,int8)")` == `0xf4bf21e5`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xf4bf21e5_u32>>::SELECTOR,
        [0xf4, 0xbf, 0x21, 0xe5],
    );

    // ink! `message_4` user-provided
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x12345678_u32>>::SELECTOR,
        0x12345678_u32.to_be_bytes(),
    );
    // `keccak256("message_4(bytes)")` == `0x0e59eb1b`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x0e59eb1b_u32>>::SELECTOR,
        [0x0e, 0x59, 0xeb, 0x1b],
    );

    // ink! inherent `blake2bx256(message_5)` == `0x9d94584e`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x9d94584e_u32>>::SELECTOR,
        [0x9d, 0x94, 0x58, 0x4e],
    );
    // `keccak256("message_5(string)")` == `0x596379bc`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x596379bc_u32>>::SELECTOR,
        [0x59, 0x63, 0x79, 0xbc],
    );

    // ink! inherent `blake2bx256(message_6)` == `0x5ac7babb`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x5ac7babb_u32>>::SELECTOR,
        [0x5a, 0xc7, 0xba, 0xbb],
    );
    // `keccak256("message_6(bool,string,bytes32,bytes,uint8[4],uint8[])")` ==
    // `0xc02c314c`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xc02c314c_u32>>::SELECTOR,
        [0xc0, 0x2c, 0x31, 0x4c],
    );

    // ink! inherent `blake2bx256(message_7)` == `0x09caadd8`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x09caadd8_u32>>::SELECTOR,
        [0x09, 0xca, 0xad, 0xd8],
    );
    // `keccak256("message_7(bytes32,bytes32,address)")` == `0xee34840f`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xee34840f_u32>>::SELECTOR,
        [0xee, 0x34, 0x84, 0x0f],
    );
}
