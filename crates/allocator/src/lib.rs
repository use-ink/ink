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

//! Crate providing allocator support for all Wasm compilations of ink! smart contracts.
//!
//! The allocator is a bump allocator whose goal is to have a small size footprint.
//! It never frees memory, having this logic in place would increase the size footprint
//! of each contract.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc_error_handler))]

#[cfg(not(any(feature = "std", feature = "no-allocator")))]
#[global_allocator]
static mut ALLOC: bump::BumpAllocator = bump::BumpAllocator {};

mod bump;

#[cfg(not(feature = "std"))]
mod handlers;

#[cfg(all(test, feature = "std", feature = "ink-fuzz-tests",))]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
