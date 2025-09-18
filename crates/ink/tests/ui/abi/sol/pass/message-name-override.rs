#![allow(unexpected_cfgs)]

use contract::Contract;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message, name = "myMessage")]
        pub fn message(&self) {}
    }
}

unsafe extern "Rust" {
    fn __ink_generate_solidity_metadata() -> ink::metadata::sol::ContractMetadata;
}

fn main() {
    // `keccak256("myMessage()")` == `0x1b008a9f`
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<0x1b008a9f_u32>>::SELECTOR,
        [0x1b, 0x00, 0x8a, 0x9f],
    );

    // Ensures `name` override is used in Solidity metadata.
    let metadata = unsafe { __ink_generate_solidity_metadata() };
    let message_specs = metadata.functions;
    assert_eq!(message_specs.len(), 1);
    assert_eq!(message_specs[0].name, "myMessage");
}
