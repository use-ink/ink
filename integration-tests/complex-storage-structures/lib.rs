#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod complex_structures {
    use ink::storage::{
        traits::{AutoKey, ManualKey, Storable, StorableHint, StorageKey},
        Mapping,
    };

    /// Non-packed type usage
    #[ink::storage_item(derive = false)]
    #[derive(Storable, StorableHint, StorageKey, Default, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct TokenManagement {
        balances: Balances,
        allowances: Allowances<ManualKey<100>>,
    }

    #[ink::storage_item]
    #[derive(Default, Debug)]
    pub struct Allowances<KEY: StorageKey> {
        allowances: Mapping<(AccountId, AccountId), Balance, AutoKey>,
    }

    #[derive(scale::Encode, scale::Decode, Default, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Balances {
        pub balance_state: u128,
    }

    impl<KEY: StorageKey> Allowances<KEY> {
        fn get_allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).unwrap_or(0)
        }

        fn set_allowance(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            value: Balance,
        ) {
            self.allowances.insert(&(owner, spender), &value);
        }
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        pub token_management: TokenManagement,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn increase_balances_state(&mut self, amount: u128) {
            self.token_management.balances.balance_state += amount;
        }

        #[ink(message)]
        pub fn decrease_balances_state(&mut self, amount: u128) {
            self.token_management.balances.balance_state -= amount;
        }

        #[ink(message)]
        pub fn get_balances_state(&self) -> u128 {
            self.token_management.balances.balance_state
        }

        #[ink(message)]
        pub fn get_allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.token_management
                .allowances
                .get_allowance(owner, spender)
        }

        #[ink(message)]
        pub fn set_allowance(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            value: u128,
        ) {
            self.token_management
                .allowances
                .set_allowance(owner, spender, value)
        }
    }
}
