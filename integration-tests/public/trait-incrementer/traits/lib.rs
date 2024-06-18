// Copyright (C) Use Ink (UK) Ltd.
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
#![cfg_attr(not(feature = "std"), no_std, no_main)]

//! Traits are extracted into a separate crate to show how the user can import
//! several foreign traits and implement those for the contract.

/// Allows to increment and get the current value.
#[ink::trait_definition]
pub trait Increment {
    /// Increments the current value of the implementer by one (1).
    #[ink(message)]
    fn inc(&mut self);

    /// Returns the current value of the implementer.
    #[ink(message)]
    fn get(&self) -> u64;
}

/// Allows to reset the current value.
#[ink::trait_definition]
pub trait Reset {
    /// Resets the current value to zero.
    #[ink(message)]
    fn reset(&mut self);
}
