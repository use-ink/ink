use contract::Contract;
use ink_lang as ink;

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
    assert_eq!(
        <Contract as ::ink_lang::DispatchableConstructorInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink_lang::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[0]
            },
        >>::SELECTOR,
        [0x5C, 0x5D, 0x42, 0x89],
    );
    assert_eq!(
        <Contract as ::ink_lang::DispatchableConstructorInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink_lang::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[1]
            },
        >>::SELECTOR,
        1_u32.to_be_bytes(),
    );
    assert_eq!(
        <Contract as ::ink_lang::DispatchableConstructorInfo<
            {
                <Contract as ::ink_lang::ContractDispatchableConstructors<
                    {
                        <Contract as ::ink_lang::ContractAmountDispatchables>::CONSTRUCTORS
                    },
                >>::IDS[2]
            },
        >>::SELECTOR,
        0xC0DE_CAFE_u32.to_be_bytes(),
    );
}
