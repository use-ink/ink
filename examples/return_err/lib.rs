#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod return_err {

    #[ink(storage)]
    pub struct ReturnErr {
        instantiated: bool,
    }

    /// Example of Error enum
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        Foo,
    }

    impl Default for ReturnErr {
        fn default() -> Self {
            Self { instantiated: true }
        }
    }

    /// Type alias for the contract's result type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl ReturnErr {
        /// Classic constructor that instantiated the contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Constructor that can be manually failed
        #[ink(constructor, payable)]
        pub fn another_new(fail: bool) -> Result<Self> {
            if fail {
                Err(Error::Foo)
            } else {
                Ok(Default::default())
            }
        }

        /// Checks if the contract has been instantiated
        #[ink(message)]
        pub fn is_instantiated(&self) -> bool {
            self.instantiated
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn classic_constructor_works() {
            let contract = ReturnErr::new();
            assert!(contract.is_instantiated())
        }

        #[ink::test]
        fn constructor_safely_fails() {
            let contract = ReturnErr::another_new(true);
            assert!(contract.is_err());
            assert_eq!(contract.err(), Some(Error::Foo))
        }

        #[ink::test]
        fn constructor_safely_works() {
            let contract = ReturnErr::another_new(false);
            assert!(contract.is_ok());
            assert!(contract.unwrap().is_instantiated())
        }
    }
}
