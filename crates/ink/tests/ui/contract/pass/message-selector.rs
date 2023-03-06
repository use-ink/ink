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
        <Contract as ::ink::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[0]
            },
        >>::SELECTOR,
        [0xFB, 0xAB, 0x03, 0xCE],
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[1]
            },
        >>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[2]
            },
        >>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[3]
            },
        >>::SELECTOR,
        [0xB6, 0xC3, 0x27, 0x49],
    );
}
