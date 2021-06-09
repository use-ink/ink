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

pub use self::accumulator::Accumulator;
use ink_lang as ink;

#[ink::contract]
pub mod accumulator {
    /// Holds a simple `i32` value that can be incremented and decremented.
    #[ink(storage)]
    pub struct Accumulator {
        value: i32,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        /// Mutates the internal value.
        #[ink(message)]
        pub fn inc(&mut self, by: i32) {
            self.value += by;
        }

        /// Returns the current state.
        #[ink(message)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }
}
