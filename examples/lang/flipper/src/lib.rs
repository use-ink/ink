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
    env::println,
    memory::format,
    storage,
    storage::Key,
    storage::Vec,
    storage::alloc::{
        AllocateUsing,
        BumpAlloc,
        DynAlloc,
        CellChunkAlloc,
        Initialize,
    },
};
use ink_lang::contract;

contract! {
    /// This simple dummy contract has a `bool` value that can
    /// alter between `true` and `false` using the `flip` message.
    /// Users can retrieve its current state using the `get` message.
    struct Flipper {
        /// The current state of our flag.
        value: storage::Value<bool>,
        outer_vec: storage::Vec<storage::Vec<i32>>,
    }

    impl Deploy for Flipper {
        /// Initializes our state to `false` upon deploying our smart contract.
        fn deploy(&mut self) {
            self.value.set(false)
        }
    }

    impl Flipper {
        /// Flips the current state of our smart contract.
        pub(external) fn flip(&mut self) {
            println(&format!("flip"));
            let mut alloc = unsafe {
                let mut fw_alloc = storage::alloc::BumpAlloc::from_raw_parts(Key([0x0; 32]));
                let mut dyn_alloc = storage::alloc::DynAlloc::allocate_using(&mut fw_alloc);
                dyn_alloc.initialize(());
                dyn_alloc
            };

            for _ in 0..100 {
                let mut inner_vec = unsafe {
                    Vec::<i32>::allocate_using(&mut alloc).initialize_into(())
                };

                for _ in 0..201 {
                    inner_vec.push(1);
                }
                self.outer_vec.push(inner_vec);
            }
            println(&format!("Outer Vec len after pushing: {:?}", self.outer_vec.len()));

            for _ in 0..self.outer_vec.len() {
                self.outer_vec.pop();
            }
            println(&format!("Outer Vec len after popping: {:?}", self.outer_vec.len()));

            *self.value = !*self.value;
        }

        /// Returns the current state.
        pub(external) fn get(&self) -> bool {
            println(&format!("Flipper Value: {:?}", *self.value));
            *self.value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Flipper;

    #[test]
    fn it_works() {
        let mut flipper = Flipper::deploy_mock();
        assert_eq!(flipper.get(), false);
        flipper.flip();
        assert_eq!(flipper.get(), true);
    }
}
