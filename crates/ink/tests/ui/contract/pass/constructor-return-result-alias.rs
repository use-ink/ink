#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        Foo,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Result<Self> {
            Err(Error::Foo)
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

use ink::metadata::InkProject;

fn generate_metadata() -> InkProject {
    extern "Rust" {
        fn __ink_generate_metadata() -> InkProject;
    }

    unsafe { __ink_generate_metadata() }
}

fn main() {
    let metadata = generate_metadata();

    let constructor = metadata.spec().constructors().iter()
        .next()
        .unwrap();

    assert_eq!("constructor", constructor.label());
}
