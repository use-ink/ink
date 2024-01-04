#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod non_fallible_api {
    use ink::{
        prelude::string::String,
        storage::{
            Lazy,
            Mapping,
        },
    };

    #[ink(storage)]
    pub struct NonFallibleAPI {
        map_1: Mapping<String, String>,
        lazy_1: Lazy<String>,
    }

    impl NonFallibleAPI {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                map_1: Mapping::new(),
                lazy_1: Lazy::new(),
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

            // Lazy
            let _ = self.lazy_1.get();
            self.lazy_1.set(&a);
        }
    }
}

fn main() {}
