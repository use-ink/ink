#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod caller {
    use dyn_traits::Increment;

    /// The caller of the incrementer smart contract.
    #[ink(storage)]
    pub struct Caller {
        incrementer: ink::contract_ref!(Increment),
    }

    impl Caller {
        /// Creates a new caller smart contract around the `incrementer` account id.
        #[ink(constructor)]
        pub fn new(incrementer: AccountId) -> Self {
            Self {
                incrementer: incrementer.into(),
            }
        }
    }

    impl Increment for Caller {
        #[ink(message)]
        fn inc(&mut self) {
            self.incrementer.inc()
        }

        #[ink(message)]
        fn get(&self) -> u64 {
            self.incrementer.get()
        }
    }
}
