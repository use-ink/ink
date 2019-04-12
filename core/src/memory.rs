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

//! Data structures to operate on main memory.
//!
//! These definitions are useful since we are operating in a `no_std` environment.

#[cfg(not(feature = "std"))]
mod no_std_defs {
    pub use alloc::{
        borrow,
        boxed,
        string,
        vec,
        format,
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
        string,
        vec,
        format,
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
