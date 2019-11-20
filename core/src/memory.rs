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

//! Data structures to operate on main memory.
//!
//! These definitions are useful since we are operating in a `no_std` environment.

#[cfg(not(feature = "std"))]
mod no_std_defs {
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

#[cfg(feature = "std")]
mod std_defs {
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
}

#[cfg(not(feature = "std"))]
#[doc(inline)]
pub use self::no_std_defs::*;

#[cfg(feature = "std")]
#[doc(inline)]
pub use self::std_defs::*;
