#![allow(unexpected_cfgs)]

use contract::Contract;

#[ink::contract]
mod contract {
    #[ink(storage)]
    pub struct Contract {}

    impl Contract {
        #[ink(constructor, name = "myConstructor")]
        pub fn constructor() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn message(&self) {}
    }
}

unsafe extern "Rust" {
    fn __ink_generate_metadata() -> ink::metadata::InkProject;
}

fn main() {
    // Constructor selector and selector id both use the name override.
    // `blake2b("myConstructor")` == `0xca295c67`
    const CTOR_ID: u32 = ::ink::selector_id!("myConstructor");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<CTOR_ID>>::SELECTOR.unwrap(),
        [0xca, 0x29, 0x5c, 0x67],
    );

    // Ensures `name` override is used in ink! metadata.
    let metadata = unsafe { __ink_generate_metadata() };
    let constructor_specs = metadata.spec().constructors();
    assert_eq!(constructor_specs.len(), 1);
    assert_eq!(*constructor_specs[0].label(), "myConstructor");
}
