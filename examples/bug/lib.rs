// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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
#![allow(clippy::new_without_default)]

use ink_lang as ink;

#[ink::contract]
mod bug {
    use ink_storage::Lazy;

    #[ink(storage)]
    pub struct Bug {
        opt: Option<u32>,
        some_val: Lazy<u32>,
    }

    impl Bug {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                opt: None,
                some_val: Lazy::new(27),
            }
        }

        #[ink(message)]
        pub fn bug(&mut self) -> Result<(), ()> {
            ink_env::debug_println("get some_val");
            let _ = *self.some_val;

            ink_env::debug_println("set opt");
            self.opt = Some(13);

            ink_env::debug_println("end");
            Ok(())
        }
    }
}
