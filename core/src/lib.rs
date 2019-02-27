// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(
    not(feature = "std"),
    feature(alloc, core_intrinsics, lang_items, alloc_error_handler,)
)]

#[cfg(not(feature = "std"))]
// #[macro_use]
extern crate alloc;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(all(test, feature = "std"))]
#[macro_use]
mod test_utils;

#[cfg(not(feature = "std"))]
mod panic_handler;

mod byte_utils;
pub mod env;
pub mod hash;
pub mod memory;
pub mod storage;
