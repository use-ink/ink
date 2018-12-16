//! Provides low-level primitives to operate on chunks of cells.

pub mod error;

mod raw_chunk;
mod typed_chunk;
mod mut_chunk;

pub(crate) use self::{
	raw_chunk::RawChunkCell,
	typed_chunk::TypedChunkCell,
};

pub use self::{
	raw_chunk::RawChunk,
	typed_chunk::TypedChunk,
	mut_chunk::MutChunk,
};
