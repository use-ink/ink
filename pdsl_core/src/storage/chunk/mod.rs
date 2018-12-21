//! Provides low-level primitives to operate on chunks of cells.

mod raw_chunk;
mod typed_chunk;
mod sync_chunk;

pub(crate) use self::{
	raw_chunk::RawChunkCell,
	typed_chunk::TypedChunkCell,
};

pub use self::{
	raw_chunk::RawChunk,
	typed_chunk::TypedChunk,
	sync_chunk::SyncChunk,
};
