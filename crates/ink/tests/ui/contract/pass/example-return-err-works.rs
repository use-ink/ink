use return_err::{
    Error,
    ReturnErr,
};

#[ink::contract]
mod return_err {

    #[ink(storage)]
    #[derive(Default)]
    pub struct ReturnErr {
        count: i32,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        Foo,
    }

    impl ReturnErr {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(constructor, payable)]
        pub fn another_new(fail: bool) -> Result<Self, Error> {
            if fail {
                Err(Error::Foo)
            } else {
                Ok(Default::default())
            }
        }

        #[ink(message)]
        pub fn get_count(&self) -> i32 {
            self.count
        }

        #[ink(message)]
        pub fn incr(&mut self, n: i32) {
            self.count += n;
        }
    }
}

fn main() {
    let contract = ReturnErr::another_new(true);
    assert!(contract.is_err());
    assert_eq!(contract.err(), Some(Error::Foo));

    let contract = ReturnErr::another_new(false);
    assert!(contract.is_ok());
    let mut contract = contract.unwrap();
    contract.incr(-5);
    assert_eq!(contract.get_count(), -5);
}
