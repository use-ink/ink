use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        Lazy,
    };

    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Lazy<Balance>,
        balances: StorageHashMap<AccountId, Balance>,
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transferred {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        amount: Balance,
    }

    #[ink(event)]
    pub struct Approved {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, initial_supply);
            let instance = Self {
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: Default::default(),
            };
            Self::env().emit_event(Transferred {
                from: None,
                to: Some(caller),
                amount: initial_supply,
            });
            instance
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> bool {
            let from = self.env().caller();
            self.transfer_from_to(from, to, amount)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, amount: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), amount);
            self.env().emit_event(Approved {
                owner,
                spender,
                amount,
            });
            true
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> bool {
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < amount {
                return false
            }
            self.allowances.insert((from, caller), allowance - amount);
            self.transfer_from_to(from, to, amount)
        }

        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
        ) -> bool {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < amount {
                return false
            }
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(from.clone(), from_balance - amount);
            self.balances.insert(to.clone(), to_balance + amount);
            self.env().emit_event(Transferred {
                from: Some(from),
                to: Some(to),
                amount,
            });
            true
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(
            &self,
            owner: &AccountId,
            spender: &AccountId,
        ) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }
    }
}

fn main() {}
