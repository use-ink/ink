// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Facilities to allocate and deallocate contract storage dynamically.

mod bump_alloc;
mod dyn_alloc;
mod traits;

#[cfg(test)]
mod tests;

pub use self::{
    bump_alloc::BumpAlloc,
    dyn_alloc::DynAlloc,
    traits::{
        Allocate,
        AllocateUsing,
        Allocator,
        Initialize,
    },
};

pub use ink_core_derive::AllocateUsing;
