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
mod flipper {
    use ink_core::storage;

    #[ink(storage)]
    struct Flipper {
        value: storage::Value<bool>,
    }

    impl Flipper {
        #[ink(constructor)]
        fn new(&mut self, init_value: bool) {
            self.value.set(init_value);
        }

        #[ink(constructor)]
        fn default(&mut self) {
            self.new(false)
        }

        #[ink(message)]
        fn flip(&mut self) {
            *self.value = !self.get();
        }

        #[ink(message)]
        fn get(&self) -> bool {
            *self.value
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn default_works() {
            let flipper = Flipper::default();
            assert_eq!(flipper.get(), false);
        }

        #[test]
        fn it_works() {
            let mut flipper = Flipper::new(false);
            assert_eq!(flipper.get(), false);
            flipper.flip();
            assert_eq!(flipper.get(), true);
        }
    }
}
