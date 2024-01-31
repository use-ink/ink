#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink_primitives::AccountId;

pub type TyAlias1 = AccountId;
pub type TyAlias2 = TyAlias1;

#[ink::contract]
pub mod non_fallible_api {
    use crate::TyAlias2;
    use ink::{
        prelude::string::String,
        storage::{
            Lazy,
            Mapping,
            StorageVec,
        },
    };

    #[ink(storage)]
    pub struct NonFallibleAPI {
        map_1: Mapping<AccountId, AccountId>,
        map_2: Mapping<AccountId, [AccountId; 1]>,
        map_3: Mapping<AccountId, (AccountId, AccountId)>,
        lazy_1: Lazy<AccountId>,
        lazy_2: Lazy<TyAlias2>,
        lazy_3: Lazy<String>,
        vec_1: StorageVec<AccountId>,
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
                lazy_3: Lazy::new(),
                vec_1: StorageVec::new(),
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

            // StorageVec
            let _ = self.vec_1.try_peek();
            let _ = self.vec_1.try_get(0);
            let _ = self.vec_1.try_set(0, &a);
            let _ = self.vec_1.try_pop();
            let _ = self.vec_1.try_push(&a);
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

            // StorageVec
            let _ = self.vec_1.peek();
            let _ = self.vec_1.get(0);
            self.vec_1.set(0, &a);
            let _ = self.vec_1.pop();
            self.vec_1.push(&a);
        }

        // Check if local suppressions work
        #[ink(message)]
        pub fn suppressions(&mut self, a: String) {
            #[cfg_attr(dylint_lib = "ink_linting", allow(non_fallible_api))]
            self.lazy_3.set(&a);
        }
    }
}

fn main() {}
