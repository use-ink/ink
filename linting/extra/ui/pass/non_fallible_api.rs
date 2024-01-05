#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink_primitives::AccountId;

pub type TyAlias1 = AccountId;
pub type TyAlias2 = TyAlias1;

#[ink::contract]
pub mod non_fallible_api {
    use crate::TyAlias2;
    use ink::storage::{
        Lazy,
        Mapping,
    };

    #[ink(storage)]
    pub struct NonFallibleAPI {
        map_1: Mapping<AccountId, AccountId>,
        map_2: Mapping<AccountId, [AccountId; 1]>,
        map_3: Mapping<AccountId, (AccountId, AccountId)>,
        lazy_1: Lazy<AccountId>,
        lazy_2: Lazy<TyAlias2>,
    }

    impl NonFallibleAPI {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                map_1: Mapping::new(),
                map_2: Mapping::new(),
                map_3: Mapping::new(),
                lazy_1: Lazy::new(),
                lazy_2: Lazy::new(),
            }
        }

        // Don't generate warnings when using the fallible API
        #[ink(message)]
        pub fn fallible(&mut self, a: AccountId, b: AccountId) {
            // Mapping
            let _ = self.map_1.try_insert(a, &b);
            let _ = self.map_1.try_get(a);
            let _ = self.map_1.try_take(a);

            // Lazy
            let _ = self.lazy_1.try_get();
            let _ = self.lazy_1.try_set(&a);
        }

        // Don't raise warnings when using non-fallible API with argument which encoded
        // size is statically known.
        #[ink(message)]
        pub fn non_fallible_statically_known(&mut self, a: AccountId, b: AccountId) {
            // Mapping
            let _ = self.map_1.insert(a, &b);
            let _ = self.map_1.get(a);
            let _ = self.map_1.take(a);
            let _ = self.map_2.insert(a, &[b; 1]);
            let _ = self.map_3.insert(a, &(b, b));

            // Lazy
            let _ = self.lazy_1.get();
            self.lazy_1.set(&a);
            let _ = self.lazy_2.get();
            self.lazy_2.set(&a);
        }
    }
}

fn main() {}
