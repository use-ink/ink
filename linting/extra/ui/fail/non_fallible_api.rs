#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub type TyAlias1 = ink::prelude::vec::Vec<i32>;
pub type TyAlias2 = TyAlias1;

#[ink::contract]
pub mod non_fallible_api {
    use crate::TyAlias2;
    use ink::{
        prelude::{
            string::String,
            vec::Vec,
        },
        storage::{
            Lazy,
            Mapping,
            StorageVec,
        },
    };

    #[ink(storage)]
    pub struct NonFallibleAPI {
        map_1: Mapping<String, String>,
        map_2: Mapping<i32, TyAlias2>,
        lazy_1: Lazy<String>,
        lazy_2: Lazy<(String, String)>,
        vec_1: StorageVec<String>,
    }

    impl NonFallibleAPI {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                map_1: Mapping::new(),
                map_2: Mapping::new(),
                lazy_1: Lazy::new(),
                lazy_2: Lazy::new(),
                vec_1: StorageVec::new(),
            }
        }

        // Raise warnings when using non-fallible API with argument which encoded size is
        // statically unknown.
        #[ink(message)]
        pub fn non_fallible_not_statically_known(&mut self, a: String, b: String) {
            // Mapping
            let _ = self.map_1.insert(a.clone(), &b);
            let _ = self.map_1.get(a.clone());
            let _ = self.map_1.take(a.clone());
            let mut v = Vec::new();
            v.push(42);
            let _ = self.map_2.insert(42, &v);

            // Lazy
            let _ = self.lazy_1.get();
            self.lazy_1.set(&a);
            self.lazy_2.set(&(a.clone(), a.clone()));

            // StorageVec
            let _ = self.vec_1.peek();
            let _ = self.vec_1.get(0);
            self.vec_1.set(0, &a.clone());
            let _ = self.vec_1.pop();
            self.vec_1.push(&a.clone());
        }
    }
}

fn main() {}
