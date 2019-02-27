// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of pDSL.
//
// pDSL is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// pDSL is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with pDSL.  If not, see <http://www.gnu.org/licenses/>.

//! Provides low-level primitives to operate on chunks of cells.

mod raw_chunk;
mod sync_chunk;
mod typed_chunk;

pub(crate) use self::raw_chunk::RawChunkCell;

pub use self::{
    raw_chunk::RawChunk,
    sync_chunk::SyncChunk,
    typed_chunk::TypedChunk,
};
