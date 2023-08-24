#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod delegatee {
    use ink::storage::{Mapping, traits::ManualKey};
    #[ink(storage)]
    pub struct Delegatee {
        addresses: Mapping<AccountId, i32, ManualKey<0x23>>,
        counter: i32,
    }

    impl Delegatee {
        /// Creates a new delegator smart contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            let v = Mapping::new();
            Self {
                addresses: v,
                counter: init_value,
            }
        }

        /// Increments the current value.
        #[ink(message)]
        pub fn inc(&mut self) {
            self.counter += 2;
        }


        /// Adds current value of counter to the `addresses`
        #[ink(message)]
        pub fn append_address_value(&mut self) {
            let caller = self.env().caller();
            self.addresses.insert(caller, &self.counter);
        }
    }
}
