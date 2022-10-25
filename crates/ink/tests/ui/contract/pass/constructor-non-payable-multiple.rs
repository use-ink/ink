#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor, selector = 0)]
        pub fn constructor_0() -> Self {
            Self {}
        }

        #[ink(constructor, selector = 1)]
        pub fn constructor_1() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

use contract::Contract;

fn main() {
    assert!(!<Contract as ::ink::traits::DispatchableConstructorInfo<0>>::PAYABLE);
    assert!(!<Contract as ::ink::traits::DispatchableConstructorInfo<1>>::PAYABLE);
}
