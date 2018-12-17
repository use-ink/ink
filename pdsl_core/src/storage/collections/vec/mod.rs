//! A contiguous growable array type, written `Vec<T>` but pronounced 'vector'.

#[cfg(all(test, feature = "test-env"))]
mod tests;

mod vec;

pub use self::vec::{
	Vec,
	Iter,
};
