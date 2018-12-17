//! A hash map implemented with quadratic probing.
//!
//! Stores its elements in the contract's storage
//! and operates directly on it.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod hash_map;

pub use self::hash_map::{
	HashMap,
};
