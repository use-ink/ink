#![allow(unexpected_cfgs)]

pub struct Error;

impl ink::primitives::sol::SolCustomError for Error {
    const NAME: &'static str = "Error";
    type Params = ();

    fn from_params(_: Self::Params) -> Self {
        Self
    }

    fn to_params(&self) -> Self::Params {}
}

ink::primitives::impl_sol_error_codec!(Error);

#[ink::contract]
mod contract {
    use super::Error;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_no_return(&self) {}

        #[ink(message)]
        pub fn message_return_non_result(&self) -> bool {
            true
        }

        #[ink(message)]
        pub fn message_return_result(&self) -> Result<u8, Error> {
            Ok(0)
        }
    }
}

fn main() {}
