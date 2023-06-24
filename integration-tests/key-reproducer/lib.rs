#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod key_reproducer {
    use ink::storage::{Mapping, traits::ManualKey};
    #[ink(storage)]
    pub struct KeyReproducer {
        values: Mapping<AccountId, Balance, ManualKey<1>>,
        value: bool,
    }

    impl KeyReproducer {
        /// Creates a new flipper smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            let v = Mapping::new();
            Self { value: init_value, values: v }
        }

        /// Creates a new flipper smart contract initialized to `false`.
        #[ink(constructor)]
        pub fn new_default() -> Self {
            Self::new(Default::default())
        }

        /// Flips the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn flip(&mut self) {
            self.value = !self.value;
        }

        #[ink(message)]
        pub fn add_value(&mut self) {
            self.values.insert(self.env().caller(), &100);
        }

        #[ink(message)]
        pub fn read_value(&self) -> Option<Balance> {
            self.values.get(self.env().caller())
        }

        /// Returns the current value of the Flipper's boolean.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }
    }
}
