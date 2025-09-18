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
    fn __ink_generate_metadata() -> ink::metadata::InkProject;
}

fn main() {
    // Message selector and selector id both use the name override.
    // `blake2b("myMessage")` == `0x6fdbfc04`
    const MESSAGE_ID: u32 = ::ink::selector_id!("myMessage");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableMessageInfo<MESSAGE_ID>>::SELECTOR,
        [0x6f, 0xdb, 0xfc, 0x04],
    );

    // Ensures `name` override is used in ink! metadata.
    let metadata = unsafe { __ink_generate_metadata() };
    let message_specs = metadata.spec().messages();
    assert_eq!(message_specs.len(), 1);
    assert_eq!(*message_specs[0].label(), "myMessage");
}
