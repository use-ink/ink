//! Low-level abstraction around the contract storage.
//!
//! Users should avoid using these abstractions directly
//! and instead prefer using higher level abstractions,
//! for example the ones that can be found in the `collections`
//! crate module.

mod key;
mod stored;
mod synced;
mod synced_chunk;

pub use self::{
	key::{
		Key,
	},
	stored::{
		Stored,
	},
	synced::{
		Synced,
		SyncedRef,
	},
	synced_chunk::{
		SyncedChunk,
	}
};
