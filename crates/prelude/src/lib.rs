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

//! Data structures to operate on contract memory during contract execution.
//!
//! These definitions are useful since we are operating in a `no_std` environment
//! and should be used by all ink! crates instead of directly using `std` or `alloc`
//! crates. If needed we shall instead enhance the exposed types here.
//!
//! The `ink_prelude` crate guarantees a stable interface between `std` and `no_std` mode.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use cfg_if::cfg_if;

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
