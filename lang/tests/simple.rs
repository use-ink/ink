// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

#![feature(const_str_as_bytes)]

use pdsl_core::storage;
use pdsl_lang::contract;

contract! {
    /// A simple contract that has a value that can be
    /// incremented, returned and compared.
    struct Incrementer {
        /// The internal value.
        value: storage::Value<u32>,
    }

    impl Deploy for Incrementer {
        /// Automatically called when the contract is deployed.
        fn deploy(&mut self, init_value: u32) {
            self.value.set(init_value)
        }
    }

    impl Incrementer {
        /// Increments the internal counter.
        pub(external) fn inc(&mut self, by: u32) {
            self.value += by
        }

        /// Returns the internal counter.
        pub(external) fn get(&self) -> u32 {
            *self.value
        }

        /// Returns `true` if `x` is greater than the internal value.
        pub(external) fn compare(&self, x: u32) -> bool {
            x > *self.value
        }
    }
}
