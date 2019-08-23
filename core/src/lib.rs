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

//! The `ink_core` utilities used by all ink! smart contracts.
//!
//! Mainly provides entities to work on a contract's storage
//! as well as high-level collections on top of those.
//! Also provides environmental utilities, such as storage allocators,
//! FFI to interface with SRML contracts and a primitive blockchain
//! emulator for simple off-chain testing.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    bad_style,
	const_err,
	dead_code,
	improper_ctypes,
	legacy_directory_ownership,
	non_shorthand_field_patterns,
	no_mangle_generic_items,
	overflowing_literals,
	path_statements,
	patterns_in_fns_without_body,
	plugin_as_library,
	private_in_public,
	safe_extern_statics,
	unconditional_recursion,
	unions_with_drop_fields,
	unused,
	unused_allocation,
	unused_comparisons,
	unused_parens,
	while_true,
	// missing-copy-implementations,
	// missing_docs,
	trivial_casts,
	trivial_numeric_casts,
	unused_extern_crates,
	// unused_import_braces,
	// unused_qualifications,
	// unused_results,
)]

// This extern crate definition is required since otherwise rustc
// is not recognizing its allocator and panic handler definitions.
#[cfg(not(feature = "std"))]
extern crate ink_alloc;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(all(test, feature = "std"))]
mod test_utils;

mod byte_utils;
pub mod env;
pub mod memory;
pub mod storage;
