use contract::Contract;
#[ink::contract(abi_encoding = "rlp")]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[ink::trait_definition]
    pub trait Messages {
        #[ink(message, selector = 1)]
        fn message_1(&self);
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

    impl Messages for Contract {
        #[ink(message, selector = 1)]
        fn message_1(&self) {}
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
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<1_u32>>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xC0DE_CAFE_u32>>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );

    // manually calculated "message_3"
    const INHERENT_ID_RLP: u32 = 0x0cd0f0f1;
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<INHERENT_ID_RLP>>::SELECTOR,
        [0x0C, 0xD0, 0xF0, 0xF1],
    );

    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x12345678_u32>>::SELECTOR,
        0x12345678_u32.to_be_bytes(),
    );
}