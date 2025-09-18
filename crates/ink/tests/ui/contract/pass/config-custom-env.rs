#![allow(unexpected_cfgs)]

#[derive(Clone)]
pub struct CustomEnv;

impl ink_env::Environment for CustomEnv {
    const NATIVE_TO_ETH_RATIO: u32 = 100_000_000;
    type AccountId = [u8; 32];
    type Balance = u64;
    type Hash = [u8; 32];
    type Timestamp = u64;
    type BlockNumber = u64;
    type EventRecord = ();
}

#[ink::contract(env = super::CustomEnv)]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

fn main() {}
