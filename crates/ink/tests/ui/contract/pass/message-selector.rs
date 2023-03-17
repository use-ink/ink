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

        #[ink(message, selector = _)]
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
    const TRAIT_ID: u32 = ::ink::selector_id!("Messages::message_0");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<TRAIT_ID>>::SELECTOR,
        [0xFB, 0xAB, 0x03, 0xCE],
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<1_u32>>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0xC0DE_CAFE_u32>>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
    const INHERENT_ID: u32 = ::ink::selector_id!("message_3");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<INHERENT_ID>>::SELECTOR,
        [0xB6, 0xC3, 0x27, 0x49],
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x12345678_u32>>::SELECTOR,
        0x12345678_u32.to_be_bytes(),
    );
}
