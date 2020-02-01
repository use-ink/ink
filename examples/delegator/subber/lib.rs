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
mod subber {
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_core::storage;
    use accumulator::Accumulator;

    /// Decreases the underlying accumulator's value.
    #[ink(storage)]
    struct Subber {
        /// The accumulator to store the value.
        accumulator: storage::Value<accumulator::Accumulator>,
    }

    impl Subber {
        /// Creates a new subber from the given accumulator.
        #[ink(constructor)]
        fn new(&mut self, accumulator: Accumulator) {
            self.accumulator.set(accumulator)
        }

        /// Decreases the accumulator's value by some amount.
        #[ink(message)]
        fn dec(&mut self, by: i32) {
            self.accumulator.inc(-by)
        }
    }
}

pub use crate::subber::Subber;
