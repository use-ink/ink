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

fn main() {
    // Constructor selector and selector id both use the name override.
    // `blake2b("myConstructor")` == `0xca295c67`
    const CTOR_ID: u32 = ::ink::selector_id!("myConstructor");
    assert_eq!(
        <Contract as ::ink::reflect::DispatchableConstructorInfo<CTOR_ID>>::SELECTOR.unwrap(),
        [0xca, 0x29, 0x5c, 0x67],
    );
}
