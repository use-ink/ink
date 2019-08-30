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

use ink_core::{
    memory::format,
    storage,
};
use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    /// Increments the accumulator's value.
    struct Adder {
        /// The accumulator to store values.
        accumulator: storage::Value<accumulator::Accumulator>,
    }

    impl Deploy for Adder {
        fn deploy(&mut self, accumulator: AccountId) {
            self.accumulator.set(accumulator::Accumulator::from_account_id(accumulator));
        }
    }

    impl Adder {
        /// Flips the current state of our smart contract.
        pub(external) fn inc(&mut self, by: i32) {
            self.accumulator.inc(by);
        }
    }
}
