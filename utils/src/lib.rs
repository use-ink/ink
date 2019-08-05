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

//! Utilities in use by ink!.
//!
//! These are kept separate from ink core utilities to allow for more dynamic inter-crate dependencies.
//! The main problem is that today Cargo manages crate features on a per-crate basis instead of
//! a per-crate-target basis thus making dependencies from `ink_lang` (or others) to `ink_core` impossible.
//!
//! By introducing `ink_utils` we have a way to share utility components between `ink_core` and
//! other parts of the framework, like `ink_lang`.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod hash;
