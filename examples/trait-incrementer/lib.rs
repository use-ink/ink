// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

/// Allows to increment and get the current value.
#[ink::trait_definition]
pub trait Increment {
    /// Increments the current value of the implementer by 1.
    #[ink(message)]
    fn inc(&mut self);

    /// Returns the current value of the implementer.
    #[ink(message)]
    fn get(&self) -> u64;
}

/// Allows to reset the current value.
#[ink::trait_definition]
pub trait Reset {
    /// Increments the current value of the implementer by 1.
    #[ink(message)]
    fn reset(&mut self);
}

#[ink::contract]
pub mod incrementer {
    use super::{
        Increment,
        Reset,
    };

    #[ink(storage)]
    pub struct Incrementer {
        value: u64,
    }

    impl Incrementer {
        /// Creates a new incrementer smart contract initialized with `0`.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                value: Default::default(),
            }
        }

        /// Increases the value of the incrementer by the given delta.
        #[ink(message)]
        pub fn inc_by(&mut self, delta: u64) {
            self.value += delta;
        }
    }

    impl Increment for Incrementer {
        #[ink(message)]
        fn inc(&mut self) {
            self.inc_by(1)
        }

        #[ink(message)]
        fn get(&self) -> u64 {
            self.value
        }
    }

    impl Reset for Incrementer {
        #[ink(message)]
        fn reset(&mut self) {
            self.value = 0;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let incrementer = Incrementer::new();
            assert_eq!(incrementer.get(), 0);
        }

        #[test]
        fn it_works() {
            let mut incrementer = Incrementer::new();
            // Can call using universal call syntax using the trait.
            assert_eq!(<Incrementer as Increment>::get(&incrementer), 0);
            <Incrementer as Increment>::inc(&mut incrementer);
            // Normal call syntax possible to as long as the trait is in scope.
            assert_eq!(incrementer.get(), 1);
        }
    }
}
