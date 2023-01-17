#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod complex_structures {
    use ink::storage::{
        traits::{
            AutoKey,
            ManualKey,
            Packed,
            Storable,
            StorableHint,
            StorageKey,
        },
        Mapping,
    };

    /// Non-packed type with
    #[ink::storage_item(derive = false)]
    #[derive(Storable, StorableHint, StorageKey, Default, Debug)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct TokenManagement<
        KEY: StorageKey,
        T: Packed + BalancesStateManagement = Balances,
    > {
        balances_state: T,
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

    pub trait BalancesStateManagement {
        fn increase_balance_state(&mut self, amount: u128);
        fn decrease_balance_state(&mut self, amount: u128);
        fn get_balance_state(&self) -> u128;
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

    impl BalancesStateManagement for Balances {
        fn increase_balance_state(&mut self, amount: u128) {
            self.balance_state += amount;
        }

        fn decrease_balance_state(&mut self, amount: u128) {
            self.balance_state -= amount;
        }

        fn get_balance_state(&self) -> u128 {
            self.balance_state
        }
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        pub token_management: TokenManagement<ManualKey<123>>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn increase_balances_state(&mut self, amount: u128) {
            self.token_management
                .balances_state
                .increase_balance_state(amount);
        }

        #[ink(message)]
        pub fn decrease_balances_state(&mut self, amount: u128) {
            self.token_management
                .balances_state
                .decrease_balance_state(amount);
        }

        #[ink(message)]
        pub fn get_balances_state(&self) -> u128 {
            self.token_management.balances_state.get_balance_state()
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
