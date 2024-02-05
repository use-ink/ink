#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod other_contract {

    #[ink(storage)]
    pub struct OtherContract {
        value: bool,
    }

    impl OtherContract {
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
