#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::error]
#[derive(Debug, PartialEq, Eq)]
/// Equivalent to multiple Solidity custom errors, one for each variant.
pub enum Error {
    /// Error when `value > 100`
    TooLarge,
    /// Error when `value == self.value`
    NoChange,
}

#[ink::contract]
pub mod fallible_setter {
    use super::Error;

    #[ink(storage)]
    pub struct FallibleSetter {
        value: u8,
    }

    impl FallibleSetter {
        /// Creates a new fallible setter smart contract initialized with the given value.
        /// Returns an error if `init_value > 100`.
        #[ink(constructor)]
        pub fn new(init_value: u8) -> Result<Self, Error> {
            if init_value > 100 {
                return Err(Error::TooLarge)
            }
            Ok(Self { value: init_value })
        }

        /// Sets the value of the FallibleSetter's `u8`.
        /// Returns an appropriate error if any of the following is true:
        /// - `value == self.value`
        /// - `init_value > 100`
        #[ink(message)]
        pub fn try_set(&mut self, value: u8) -> Result<(), Error> {
            if self.value == value {
                return Err(Error::NoChange);
            }

            if value > 100 {
                return Err(Error::TooLarge);
            }

            self.value = value;
            Ok(())
        }

        /// Returns the current value of the FallibleSetter's `u8`.
        #[ink(message)]
        pub fn get(&self) -> u8 {
            self.value
        }
    }
}

#[cfg(test)]
mod tests;