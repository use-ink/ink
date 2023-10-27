#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::associated_messages::{
    AssociatedMessages,
    AssociatedMessagesRef,
};

#[ink::contract]
mod associated_messages {
    #[ink(storage)]
    pub struct AssociatedMessages {
        value: i32,
    }

    impl AssociatedMessages {
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        #[ink(message)]
        pub fn associated_message(value: i32) -> i32 {
            ink_env::debug_println!("associated_message: {}", value);
            value
        }

        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value = self.value.checked_add(by).unwrap();
        }

        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn default_works() {
            let contract = AssociatedMessages::new_default();
            assert_eq!(contract.get(), 0);
        }

        #[ink::test]
        fn it_works() {
            let mut contract = AssociatedMessages::new(42);
            assert_eq!(contract.get(), 42);
            contract.inc(5);
            assert_eq!(contract.get(), 47);
            contract.inc(-50);
            assert_eq!(contract.get(), -3);
        }
    }
}
