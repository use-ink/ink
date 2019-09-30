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

//! Facilities to allocate and deallocate contract storage dynamically.

mod bump_alloc;
mod dyn_alloc;
mod traits;

#[cfg(all(test, feature = "test-env"))]
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
