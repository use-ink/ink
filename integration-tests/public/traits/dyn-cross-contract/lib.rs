//! This crate contains the `Caller` contract with no functionality except forwarding
//! all calls to the `trait_incrementer::Incrementer` contract.
//!
//! The `Caller` doesn't use the `trait_incrementer::IncrementerRef`. Instead,
//! all interactions with the `Incrementer` is done through the wrapper from
//! `ink::contract_ref_from_path!` and the trait `dyn_traits::Increment`.
#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![allow(clippy::new_without_default)]

#[ink::contract]
pub mod caller {
    use dyn_traits::Increment;

    /// The caller of the incrementer smart contract.
    #[ink(storage)]
    pub struct Caller {
        /// Here we accept a type which implements the `Incrementer` ink! trait.
        incrementer: ink::contract_ref_from_path!(Increment),
    }

    impl Caller {
        /// Creates a new caller smart contract around the `incrementer` account id.
        #[ink(constructor)]
        pub fn new(incrementer: Address) -> Self {
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

#[cfg(test)]
mod tests;