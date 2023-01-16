#![cfg_attr(not(feature = "std"), no_std)]

pub use self::complex_structures::{
    Balances,
    Contract,
    ContractRef,
};

#[ink::contract]
pub mod complex_structures {
    use ink::storage::{
        traits::{
            AutoKey,
            ManualKey,
            Packed,
            StorageKey,
        },
        Mapping,
    };

    #[ink::storage_item]
    #[derive(Default, Debug)]
    pub struct Balances<KEY: StorageKey, T: Packed = u128> {
        pub balance_state: T,
    }

    impl<KEY: StorageKey> Balances<KEY> {
        pub fn increase_state(&mut self, amount: u128) {
            self.balance_state += amount;
        }
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        pub balances: Balances<ManualKey<123>>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();

            instance.balances = Default::default();

            instance
        }

        #[ink(message)]
        pub fn increase_state(&mut self, amount: u128) {
            self.balances.increase_state(amount);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ink::{
        env::DefaultEnvironment,
        storage::traits::{
            AutoKey,
            ManualKey,
            StorageKey,
        },
    };

    #[test]
    fn keys_work() {
        assert_eq!(<Balances<AutoKey> as StorageKey>::KEY, 0);
        assert_eq!(<Balances<ManualKey<123>> as StorageKey>::KEY, 123);
        assert_eq!(<Balances{ balances_state: 1} as StorageKey>::KEY, 100);
    }
}
