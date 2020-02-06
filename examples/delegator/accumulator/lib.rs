// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

#[ink::contract(version = "0.1.0")]
mod accumulator {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage;
    
    /// Holds a simple i32 value that can be incremented and decremented.
    #[ink(storage)]
    struct Accumulator {
        value: storage::Value<i32>,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        fn new(&mut self, init_value: i32) {
            self.value.set(init_value)
        }

        /// Mutates the internal value.
        #[ink(message)]
        fn inc(&mut self, by: i32) {
            self.value += by;
        }

        /// Returns the current state.
        #[ink(message)]
        fn get(&self) -> i32 {
            *self.value
        }
    }
}

pub use crate::accumulator::Accumulator;
