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

use ink_core::{
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// A simple incrementer contract that can only increment,
    /// compare and return its internal value.
    struct Incrementer {
        /// The current value.
        value: storage::Value<u32>,
    }

    impl Deploy for Incrementer {
        /// Initializes the value to the initial value.
        fn deploy(&mut self, init_value: u32) {
            self.value.set(init_value)
        }
    }

    impl Incrementer {
        /// Flips the current state of our smart contract.
        pub(external) fn inc(&mut self, by: u32) {
            env.println(&format!("Incrementer::inc by = {:?}", by));
            self.value += by;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> u32 {
            env.println(&format!("Incrementer::get = {:?}", *self.value));
            *self.value
        }

        /// Returns `true` if the internal value is greater than or equal to the provided value.
        pub(external) fn compare(&self, with: u32) -> bool {
            env.println(&format!("Incrementer::compare self.value >= with = {:?}", *self.value >= with));
            *self.value >= with
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Incrementer;

    #[test]
    fn it_works() {
        let mut incrementer = Incrementer::deploy_mock(5);
        assert_eq!(incrementer.get(), 5);
        incrementer.inc(42);
        assert_eq!(incrementer.get(), 47);
        incrementer.inc(0);
        assert_eq!(incrementer.get(), 47);
    }
}
