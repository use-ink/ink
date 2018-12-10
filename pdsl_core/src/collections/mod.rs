//! Collections operating on contract storage.
//!
//! Collection types that safely abstract from contract storage.
//! Users are recommended to use them instead of unsafe abstractions
//! such as `Key` or `Storage`.
//!
//! Currently supported data structures are:
//!
//! - `StorageVec`: Similar to Rust's `Vec`
//! - `StorageMap`: Similar to Rust's `HashMap`
//!
//! Beware that the similarities are only meant for their respective APIs.
//! Internally they are structured completely different and may even
//! exhibit different efficiency characteristics.

pub mod storage_map;
mod storage_vec;

pub use self::{
	storage_vec::{
		StorageVec,
	},
	storage_map::{
		StorageMap,
	},
};
