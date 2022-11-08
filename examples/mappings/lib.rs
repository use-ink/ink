#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod mappings {
    use ink::storage::Mapping;

    /// A simple ERC-20 contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Mappings {
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    impl Mappings {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
            }
        }

        /// Demonstrates the usage of `Mapping::get()`.
        /// Returns the balance of a account.
        ///
        /// Returns `None` if the account is non-existent.
        #[ink(message)]
        pub fn get_balance(&self, owner: AccountId) -> Option<Balance> {
            self.balances.get(owner)
        }

        /// Demonstrates the usage of `Mappings::insert()`.
        ///
        /// Assigns the value to a given account.
        /// Returns the size of the pre-existing balance at the specified key if any.
        ///
        /// Returns `None` if the account was non-existent.
        #[ink(message)]
        pub fn insert_return_size_balance(
            &mut self,
            value: Balance,
            to: AccountId,
        ) -> Option<u32> {
            self.balances.insert(to, &value)
        }

        /// Demonstrates the usage of `Mappings::size()`.
        ///
        /// Returns the size of the pre-existing value at the specified key if any.
        ///
        /// Returns `None` if the account was non-existent.
        #[ink(message)]
        pub fn size_balance(&mut self, of: AccountId) -> Option<u32> {
            self.balances.size(of)
        }

        /// Demonstrates the usage of `Mapping::contains()`.
        ///
        /// Returns `true` if the account has any balance assigned to it.
        #[ink(message)]
        pub fn contains_balance(&self, of: AccountId) -> bool {
            self.balances.contains(of)
        }

        /// Demonstrates the usage of `Mappings::remove()`.
        ///
        /// Removes the balance entry for a given account.
        #[ink(message)]
        pub fn remove_balance(&mut self, of: AccountId) {
            self.balances.remove(of);
        }

        /// Demonstrates the usage of `Mappings::take()`.
        ///
        /// Returns the balance of a given account
        /// removing it from storage.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn take_balance(&mut self, from: AccountId) -> Option<Balance> {
            self.balances.take(from)
        }
    }

    #[cfg(test)]
    mod e2e_tests {}
}
