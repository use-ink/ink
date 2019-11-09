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

#![cfg_attr(not(feature = "std"), no_std)]
#![feature(proc_macro_hygiene)]

use ink_core::storage;
use ink_lang2 as ink;

#[ink::contract(version = "0.1.0")]
mod accumulator {
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
