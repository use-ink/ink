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

        #[ink(message, selector = 1, payable)]
        pub fn message_1(&self) {}

        #[ink(message, selector = 2)]
        pub fn message_2(&self) {}
    }
}

use contract::Contract;

fn main() {
    assert!(<Contract as ::ink_lang::reflect::DispatchableMessageInfo<1>>::PAYABLE);
    assert!(!<Contract as ::ink_lang::reflect::DispatchableMessageInfo<2>>::PAYABLE);
}
