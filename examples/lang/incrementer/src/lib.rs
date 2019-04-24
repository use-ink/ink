// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(any(test, feature = "test-env")), no_std)]

use ink_core::{
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
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
