#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod return_err {

    #[ink(storage)]
    pub struct ReturnErr {
        instantiated: bool,
    }

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

    pub type Result<T> = core::result::Result<T, Error>;

    impl ReturnErr {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(constructor, payable)]
        pub fn new2(fail: bool) -> Result<Self> {
            if fail {
                Err(Error::Foo)
            } else {
                Ok(Default::default())
            }
        }

        #[ink(message)]
        pub fn is_instantiated(&self) -> bool {
            self.instantiated
        }
    }
}
