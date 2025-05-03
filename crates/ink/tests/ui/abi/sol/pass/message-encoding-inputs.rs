#![allow(unexpected_cfgs)]

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message_0(
            &mut self,
            _input_1: i8,
            _input_2: i16,
            _input_3: i32,
            _input_4: i64,
            _input_5: i128,
            _input_6: u8,
            _input_7: u16,
            _input_8: u32,
            _input_9: u64,
            _input_10: u128,
        ) {
        }

        #[ink(message)]
        pub fn message_1(
            &mut self,
            _input_0: bool,
            _input_1: String,
            _input_2: Vec<u8>,
            _input_3: [u8; 32],
        ) {
        }

        #[ink(message)]
        pub fn message_2_primitives(
            &mut self,
            _input_0: AccountId,
            _input_1: Hash,
            _input_2: Address,
            _input_3: SolBytes<[u8; 32]>,
            _input_4: SolBytes<Vec<u8>>,
        ) {
        }
    }
}

fn main() {}
