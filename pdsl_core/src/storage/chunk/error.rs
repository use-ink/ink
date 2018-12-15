//! Errors that may be encountered while working with chunks of cells.

use std::error::Error;
use std::fmt::Display;

/// A kind of a chunk error.
#[derive(Debug, PartialEq, Eq)]
pub enum ChunkErrorKind {
	/// When trying to access out of bounds.
	AccessOutOfBounds{
		access_at: u32,
		limit_at: u32,
	},
	/// When trying to operate on a valid but empty (or uninitialized) storage slot.
	EmptyStorageSlot{
		at: u32,
	},
}

/// A chunk error.
#[derive(Debug, PartialEq, Eq)]
pub struct ChunkError {
	/// The kind of the chunk error.
	kind: ChunkErrorKind
}

/// The chunk error result type.
pub type Result<T> = std::result::Result<T, ChunkError>;

impl ChunkError {
	/// Returns the kind of the chunk error.
	pub fn kind(&self) -> &ChunkErrorKind {
		&self.kind
	}

	/// Creates an access-out-of-bounds chunk error.
	pub(crate) fn access_out_of_bounds(access_at: u32, limit_at: u32) -> Self {
		Self{ kind: ChunkErrorKind::AccessOutOfBounds{access_at, limit_at}}
	}

	/// Creates an empty-storage-slot chunk error.
	pub(crate) fn empty_slot(at: u32) -> Self {
		Self{ kind: ChunkErrorKind::EmptyStorageSlot{at}}
	}
}

impl Error for ChunkError {}

impl Display for ChunkError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self.kind() {
			ChunkErrorKind::AccessOutOfBounds{access_at, limit_at} => {
				write!(
					f,
					"[pdsl_core] Error: encountered cell chunk access out of bounds (at: {:?}, limit: {:?})",
					access_at,
					limit_at,
				)
			}
			ChunkErrorKind::EmptyStorageSlot{at} => {
				write!(
					f,
					"[pdsl_core] Error: tried to operate on a valid but empty storage slot (at: {:?})",
					at,
				)
			}
		}
	}
}
