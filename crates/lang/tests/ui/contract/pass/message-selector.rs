use contract::Contract;
use ink_lang as ink;

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
        pub fn message_0(&self) {}

        #[ink(message, selector = 1)]
        pub fn message_1(&self) {}

        #[ink(message, selector = 0xC0DE_CAFE)]
        pub fn message_2(&self) {}
    }
}

fn main() {
    assert_eq!(
        <Contract as ::ink_lang::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink_lang::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[0]
            },
        >>::SELECTOR,
        [0x5A, 0x6A, 0xC1, 0x5D],
    );
    assert_eq!(
        <Contract as ::ink_lang::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink_lang::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[1]
            },
        >>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink_lang::reflect::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::reflect::ContractDispatchableMessages<
                    {
                        <Contract as ::ink_lang::reflect::ContractAmountDispatchables>::MESSAGES
                    },
                >>::IDS[2]
            },
        >>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
}
