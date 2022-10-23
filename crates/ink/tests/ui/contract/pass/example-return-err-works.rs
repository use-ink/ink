use return_err::ReturnErr;

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
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(constructor, payable)]
        pub fn another_new(fail: bool) -> Result<Self> {
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

fn main() {
    let contract = ReturnErr::another_new(true);
    assert!(contract.is_err());
    assert_eq!(contract.err(), Some(Error::Foo));

    let contract = ReturnErr::another_new(false);
    assert!(contract.is_ok());
    assert!(contract.unwrap().is_instantiated());
}
