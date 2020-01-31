// Copyright 2018-2019 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::CacheGuard;
use crate::storage::{
    alloc::{
        Allocate,
        AllocateUsing,
    },
    chunk::TypedChunk,
    Flush,
};
#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutRange,
    StorageLayout,
};
use ink_primitives::Key;
#[cfg(feature = "ink-generate-abi")]
use type_metadata::{
    HasTypeDef,
    Metadata,
    NamedField,
    TypeDef,
    TypeDefStruct,
    TypeId,
};

/// A chunk of synchronized cells.
///
/// Provides mutable and read-optimized access to the associated contract storage slot.
///
/// # Guarantees
///
/// - `Owned`
/// - `Typed`
/// - `Opt. Reads`
/// - `Mutable`
///
/// Read more about kinds of guarantees and their effect [here](../index.html#guarantees).
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(TypeId))]
pub struct SyncChunk<T> {
    /// The underlying chunk of cells.
    chunk: TypedChunk<T>,
    /// The cached element.
    cache: CacheGuard<T>,
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasTypeDef for SyncChunk<T> {
    fn type_def() -> TypeDef {
        TypeDefStruct::new(vec![NamedField::of::<Key>("cells_key")]).into()
    }
}

impl<T> Flush for SyncChunk<T>
where
    T: scale::Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        for (n, dirty_val) in self.cache.iter_dirty() {
            match dirty_val.get_mut() {
                Some(val) => {
                    self.chunk.store(n, val);
                    val.flush();
                }
                None => self.chunk.clear(n),
            }
            dirty_val.mark_clean();
        }
    }
}

impl<T> scale::Encode for SyncChunk<T> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.chunk.encode_to(dest)
    }
}

impl<T> scale::Decode for SyncChunk<T> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        TypedChunk::decode(input).map(|typed_chunk| {
            Self {
                chunk: typed_chunk,
                cache: Default::default(),
            }
        })
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasLayout for SyncChunk<T>
where
    T: Metadata,
{
    fn layout(&self) -> StorageLayout {
        LayoutRange::chunk(self.cells_key(), T::meta_type()).into()
    }
}

impl<T> AllocateUsing for SyncChunk<T> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            chunk: TypedChunk::allocate_using(alloc),
            cache: CacheGuard::default(),
        }
    }
}

impl<T> SyncChunk<T> {
    /// Clears the cache value at position `n`.
    pub fn clear(&mut self, n: u32) {
        self.cache.update_mut(n, None);
    }

    /// Returns the underlying key to the cells.
    ///
    /// # Note
    ///
    /// This is a low-level utility getter and should
    /// normally not be required by users.
    pub fn cells_key(&self) -> Key {
        self.chunk.key()
    }
}

impl<T> SyncChunk<T>
where
    T: scale::Decode,
{
    /// Returns the value of the `n`-th cell if any.
    #[must_use]
    pub fn get(&self, n: u32) -> Option<&T> {
        match self.cache.get(n) {
            Some(cache_value) => cache_value.get(),
            None => self.cache.update(n, self.chunk.load(n)),
        }
    }

    /// Returns the value of the `n`-th cell if any.
    #[must_use]
    pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
        match self.cache.get_mut(n) {
            Some(cache_value) => cache_value.get_mut(),
            None => self.cache.update_mut(n, self.chunk.load(n)),
        }
    }

    /// Takes the value of the `n`-th cell if any.
    ///
    /// # Note
    ///
    /// Prefer using [clear](struct.SyncChunk.html#method.clear)
    /// if you are not interested in the return value since it
    /// is more efficient.
    #[must_use]
    pub fn take(&mut self, n: u32) -> Option<T> {
        match self.cache.get_mut(n) {
            Some(cache_value) => cache_value.take(),
            None => {
                self.cache.update_mut(n, None);
                self.chunk.load(n)
            }
        }
    }
}

impl<T> SyncChunk<T>
where
    T: scale::Encode,
{
    /// Sets the value of the `n`-th cell.
    pub fn set(&mut self, n: u32, val: T) {
        self.cache.update_mut(n, Some(val));
    }
}

impl<T> SyncChunk<T>
where
    T: scale::Codec,
{
    /// Replaces the value of the `n`-th cell and returns its old value if any.
    ///
    /// # Note
    ///
    /// Prefer using [set](struct.SyncChunk.html#method.set)
    /// if you are not interested in the return value since it
    /// is more efficient.
    #[must_use]
    pub fn put(&mut self, n: u32, new_val: T) -> Option<T> {
        match self.cache.get_mut(n) {
            Some(cache_value) => cache_value.put(Some(new_val)),
            None => {
                self.cache.update_mut(n, Some(new_val));
                self.chunk.load(n)
            }
        }
    }
}
