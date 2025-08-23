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

//! Data structures to operate on contract memory during contract execution.
//!
//! These definitions are useful since we are operating in a `no_std` environment
//! and should be used by all ink! crates instead of directly using `std` or `alloc`
//! crates. If needed we shall instead enhance the exposed types here.
//!
//! The `ink_prelude` crate guarantees a stable interface between `std` and `no_std` mode.

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use cfg_if::cfg_if;

/// A well known selector reserved for the message required to be defined
/// alongside a wildcard selector. See [IIP-2](https://github.com/use-ink/ink/issues/1676).
///
/// Calculated from `selector_bytes!("IIP2_WILDCARD_COMPLEMENT")`
pub const IIP2_WILDCARD_COMPLEMENT_SELECTOR: [u8; 4] = [0x9B, 0xAE, 0x9D, 0x5E];

cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::{
            borrow,
            boxed,
            format,
            string,
            vec,
        };

        /// Collection types.
        pub mod collections {
            pub use self::{
                binary_heap::BinaryHeap,
                btree_map::BTreeMap,
                btree_set::BTreeSet,
                linked_list::LinkedList,
                vec_deque::VecDeque,
                Bound,
            };
            pub use std::collections::*;
        }
    } else {
        pub use alloc::{
            borrow,
            boxed,
            format,
            string,
            vec,
        };
        //pub use core::sync::Mutex;
        //pub use sync::Mutex;

        /// Collection types.
        pub mod collections {
            pub use self::{
                BTreeMap,
                BTreeSet,
                BinaryHeap,
                LinkedList,
                VecDeque,
            };
            pub use alloc::collections::*;
            pub use core::ops::Bound;
        }
    }
}
