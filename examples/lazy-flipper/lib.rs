#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod flipper {
    use ink_storage::{
        lazy::Mapping,
        traits::SpreadAllocate,
        Lazy,
    };

    #[ink(storage)]
    #[derive(SpreadAllocate, Default)]
    pub struct Flipper {
        map: Mapping<bool, ()>,
        lazy: Lazy<()>,
    }

    impl Flipper {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::codegen::initialize_contract(|_contract: &mut Self| {})
        }

        #[ink(message)]
        pub fn get(&self) -> bool {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn it_works() {
            let flipper = Flipper::new();
            dbg!(&flipper);
            let _ = *flipper.lazy;
            // panic!()
        }
    }
}
