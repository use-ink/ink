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
    // #[derive(Default)]
    pub struct Flipper {
        map: Mapping<bool, ()>,
        map2: Mapping<bool, ()>,
        lazy: Lazy<()>,
        vec: ink_storage::collections::Vec<()>,
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
