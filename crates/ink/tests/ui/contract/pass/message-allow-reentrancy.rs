#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, selector = 1, allow_reentrancy)]
        pub fn message_1(&self) {}

        #[ink(message, selector = 2)]
        pub fn message_2(&self) {}
    }
}

use contract::Contract;

fn main() {
    assert!(<Contract as ::ink::reflect::DispatchableMessageInfo<1>>::ALLOW_REENTRANCY);
    assert!(!<Contract as ::ink::reflect::DispatchableMessageInfo<2>>::ALLOW_REENTRANCY);
}
