//! A contiguous growable array type, written `Vec<T>` but pronounced 'vector'.
//!
//! Stores its elements in the contract's storage
//! and operates directly on it.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod vec;

pub use self::vec::{
	Vec,
	Iter,
};
