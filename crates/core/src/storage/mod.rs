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

//! Core abstractions for storage manipulation. (revision 2)

pub mod alloc;
pub mod collections;
pub mod lazy;
mod memory;
mod pack;
pub mod traits;

#[cfg(test)]
mod hashmap_entry_api_tests;

#[doc(inline)]
pub use self::{
    alloc::Box,
    collections::Vec,
    lazy::Lazy,
    memory::Memory,
    pack::Pack,
};
