#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! The trait is extracted into a separate crate to show how to do cross-contract
//! calls only with traits without importing the contract.

/// Allows to increment and get the current value.
#[ink::trait_definition]
pub trait Increment {
    /// Increments the current value of the implementer by one (1).
    #[ink(message)]
    fn inc(&mut self);

    /// Returns the current value of the implementer.
    #[ink(message)]
    fn get(&self) -> u64;
}
