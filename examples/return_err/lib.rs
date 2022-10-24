#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod return_err {

    #[ink(storage)]
    #[derive(Default)]
    pub struct ReturnErr {
        count: i32,
    }

    /// Example of Error enum
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        Foo,
    }

    impl ReturnErr {
        /// Classic constructor that instantiated the contract
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        /// Constructor that can be manually failed
        #[ink(constructor, payable)]
        pub fn another_new(fail: bool) -> Result<Self, Error> {
            if fail {
                Err(Error::Foo)
            } else {
                Ok(Default::default())
            }
        }

        /// Gets the value of a counter
        #[ink(message)]
        pub fn get_count(&self) -> i32 {
            self.count
        }

        /// Changes the value of counter
        #[ink(message)]
        pub fn incr(&mut self, n: i32) {
            self.count += n;
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn classic_constructor_works() {
            let mut contract = ReturnErr::new();
            contract.incr(5);
            assert_eq!(contract.get_count(), 5)
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
            let mut contract = contract.unwrap();
            contract.incr(-5);
            assert_eq!(contract.get_count(), -5);
        }
    }
}
