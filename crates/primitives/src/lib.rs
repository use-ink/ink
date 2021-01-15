// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! Utilities in use by ink!.
//!
//! These are kept separate from ink core utilities to allow for more dynamic inter-crate dependencies.
//! The main problem is that today Cargo manages crate features on a per-crate basis instead of
//! a per-crate-target basis thus making dependencies from `ink_lang` (or others) to `ink_env` or `ink_storage` impossible.
//!
//! By introducing `ink_primitives` we have a way to share utility components between `ink_env` or `ink_storage` and
//! other parts of the framework, like `ink_lang`.

#![cfg_attr(not(feature = "std"), no_std)]

mod key;
mod key_ptr;

pub use self::{
    key::Key,
    key_ptr::KeyPtr,
};
