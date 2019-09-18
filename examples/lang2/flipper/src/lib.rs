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

#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]

use ink_core::{
    env::DefaultSrmlTypes,
    memory::format,
    storage,
};
use ink_lang as ink;

#[ink::contract(
    env = DefaultSrmlTypes,
    version = [0, 1, 0],
)]
mod flipper {
    #[ink(storage)]
    struct Flipper {
        value: storage::Value<bool>,
    }

    impl Flipper {
        /// Initializes with `false`.
        #[ink(constructor)]
        fn default(&mut self) {
            self.value.set(false)
        }

        /// Flips the boolean.
        #[ink(message)]
        fn flip(&mut self) {
            *self.value = !*self.value;
        }

        /// Returns the boolean.
        #[ink(message)]
        fn get(&self) -> bool {
            env.println(&format!("Flipper Value: {:?}", *self.value));
            *self.value
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Flipper;

//     #[test]
//     fn it_works() {
//         let mut flipper = Flipper::deploy_mock();
//         assert_eq!(flipper.get(), false);
//         flipper.flip();
//         assert_eq!(flipper.get(), true);
//     }
// }
