#![feature(proc_macro_hygiene)]

use ink_lang2 as ink;
use ink_core::{
    env2::DefaultSrmlTypes,
    storage,
};

#[ink::contract(
    env = DefaultSrmlTypes,
    version = "0.1.0",
)]
mod erc20 {
    #[ink(storage)]
    struct Erc20 {
        total_supply: storage::Value<Balance>,
        balances: storage::HashMap<AccountId, Balance>,
        allowances: storage::HashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    struct Transferred {
        #[indexed] from: Option<AccountId>,
        #[indexed] to: Option<AccountId>,
        #[indexed] amount: Balance,
    }

    #[ink(event)]
    struct Approved {
        #[indexed] owner: AccountId,
        #[indexed] spender: AccountId,
        #[indexed] amount: Balance,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(&mut self, initial_supply: Balance) {
            self.total_supply.set(initial_supply);
            self.balances.insert(self.env().caller(), initial_supply);
            self.env().emit_event(Transferred {
                from: None,
                to: Some(self.env().caller()),
                value: initial_supply
            });
        }

        #[ink(message)]
        fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, amount: Balance) -> bool {
            let from = self.env().caller();
            self.transfer_from_to(from, to, amount)
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, amount: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), amount);
            self.env().emit_event(Approved {
                owner: owner,
                spender: spender,
                amount
            });
            true
        }

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, amount: Balance) -> bool {
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < amount {
                return false
            }
            self.allowances.insert((from, caller), allowance - amount);
            self.transfer_from_to(from, to, amount)
        }

        fn transfer_from_to(&mut self, from: AccountId, to: AccountId, amount: Balance) -> bool {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < amount {
                return false;
            }
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(from.clone(), from_balance - amount);
            self.balances.insert(to.clone(), to_balance + amount);
            self.env().emit_event(Transferred { from, to, amount });
            true
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }
    }
}

fn main() {}
