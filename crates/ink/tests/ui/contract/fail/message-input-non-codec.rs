#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[::ink::scale_derive(TypeInfo)]
    pub struct NonCodecType;

    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self, _input: NonCodecType) {}
    }
}

fn main() {}
