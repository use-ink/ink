//! Data structures to operate on main memory.
//!
//! These definitions are useful since we are operating in a `no_std` environment.

#[cfg(not(feature = "std"))]
mod no_std_defs {
	pub use alloc::boxed;
	pub use alloc::vec;
	pub use alloc::string;
	pub use alloc::borrow;

	/// Collection types.
	pub mod collections {
		pub use alloc::collections::*;
		pub use hashbrown::{hash_map, hash_set};
		pub use self::{
			BinaryHeap,
			BTreeMap,
			BTreeSet,
			LinkedList,
			VecDeque,
			hash_map::HashMap,
			hash_set::HashSet,
		};
		pub use core::ops::Bound;
	}
}

#[cfg(feature = "std")]
mod std_defs {
	pub use std::boxed;
	pub use std::vec;
	pub use std::string;
	pub use std::borrow;

	/// Collection types.
	pub mod collections {
		pub use std::collections::*;
		pub use self::{
			binary_heap::BinaryHeap,
			btree_map::BTreeMap,
			btree_set::BTreeSet,
			linked_list::LinkedList,
			vec_deque::VecDeque,
			hash_map::HashMap,
			hash_set::HashSet,
			Bound,
		};
	}
}

#[cfg(not(feature = "std"))]
#[doc(inline)]
pub use self::no_std_defs::*;

#[cfg(feature = "std")]
#[doc(inline)]
pub use self::std_defs::*;
