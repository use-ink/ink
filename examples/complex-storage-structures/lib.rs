#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod complex_structures {
    use ink::storage::{
        traits::{
            AutoKey,
            ManualKey,
            Packed,
            StorageKey,
        },
        Lazy,
        Mapping,
    };

    #[ink::storage_item]
    #[derive(Default, Debug)]
    struct Balances<KEY: StorageKey, T: Packed = u128> {
        pub balance_state: T,
    }

    impl<KEY: StorageKey> Balances<KEY, T> {
        pub fn increase_state(&mut self, amount: u128) {
            self.balance_state += amount;
        }
    }

    #[ink(storage)]
    #[derive(Debug, Default)]
    pub struct Contract {
        balances: Balances<AutoKey>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn increase_state(&mut self, amount: u128) {
            self.balances.increase_state(amount);
        }
    }
}
