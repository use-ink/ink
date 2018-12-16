//! Provides low-level primitives to operate on chunks of cells.

pub mod error;

mod raw_chunk;
mod typed_chunk;
mod copy_chunk;
mod mut_chunk;

pub(crate) use self::{
	raw_chunk::RawChunkCell,
	typed_chunk::TypedChunkCell,
};

pub use self::{
	raw_chunk::RawChunk,
	typed_chunk::TypedChunk,
	copy_chunk::CopyChunk,
	mut_chunk::MutChunk,
};
