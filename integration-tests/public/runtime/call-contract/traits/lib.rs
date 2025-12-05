#![cfg_attr(not(feature = "std"), no_std, no_main)]

/// Allows to flip and get a bool value.
#[ink::trait_definition]
pub trait Flip {
    /// Flip the value of the stored `bool` from `true`
    /// to `false` and vice versa.
    #[ink(message)]
    fn flip(&mut self);

    /// Returns the current value of our `bool`.
    #[ink(message)]
    fn get(&self) -> bool;
}
