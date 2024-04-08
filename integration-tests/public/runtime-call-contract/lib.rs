#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use flipper::{
    Flipper,
    FlipperRef,
};

#[ink::contract]
mod flipper {
    #[ink(storage)]
    pub struct Flipper {
        value: bool,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }
    }

    impl flipper_traits::Flip for Flipper {
        #[ink(message)]
        fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        fn get(&self) -> bool {
            self.value
        }
    }
}

#[cfg(test)]
mod e2e_tests;
