// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

//! Low-level collections and data structures to manage storage entities in the
//! persisted contract storage.
//!
//! Users should generally avoid using these collections directly in their
//! contracts and should instead adhere to the high-level collections found
//! in [`collections`][`crate::collections`].
//! The low-level collections are mainly used as building blocks for internals
//! of other higher-level storage collections.
//!
//! These low-level collections are not aware of the elements they manage thus
//! extra care has to be taken when operating directly on them.

mod mapping;
mod storage_value;

#[doc(inline)]
pub use self::{
    mapping::Mapping,
    storage_value::StorageValue,
};
