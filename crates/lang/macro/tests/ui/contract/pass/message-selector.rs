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
        <Contract as ::ink_lang::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableMessages<
                    { <Contract as ::ink_lang::ContractAmountDispatchables>::MESSAGES },
                >>::IDS[0]
            },
        >>::SELECTOR,
        [90, 106, 193, 93],
    );
    assert_eq!(
        <Contract as ::ink_lang::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableMessages<
                    { <Contract as ::ink_lang::ContractAmountDispatchables>::MESSAGES },
                >>::IDS[1]
            },
        >>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink_lang::DispatchableMessageInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableMessages<
                    { <Contract as ::ink_lang::ContractAmountDispatchables>::MESSAGES },
                >>::IDS[2]
            },
        >>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
}
