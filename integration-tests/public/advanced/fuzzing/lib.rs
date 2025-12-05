#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod fuzz_testing {
    #[ink(storage)]
    pub struct FuzzTesting {
        value: bool,
    }

    //#[derive(PartialEq, Eq, Debug, Clone)]
    //#[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    #[derive(Clone, Debug)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub struct Point {
        pub x: i32,
        pub y: i32,
    }

    impl FuzzTesting {
        /// Creates a new contract initialized with the given value.
        #[ink(constructor)]
        pub fn new(init_value: bool) -> Self {
            Self { value: init_value }
        }

        /// Returns the current value.
        #[ink(message)]
        pub fn get(&self) -> bool {
            self.value
        }

        /// Extracts `Point.x`.
        #[ink(message)]
        pub fn extract_x(&self, pt: Point) -> i32 {
            pt.x
        }
    }
}

#[cfg(all(test, feature = "e2e-tests"))]
mod tests;