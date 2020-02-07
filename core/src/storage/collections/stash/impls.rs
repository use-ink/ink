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

use crate::{
    ink_core,
    storage::{
        self,
        alloc::{
            Allocate,
            AllocateUsing,
            Initialize,
        },
        chunk::SyncChunk,
        Flush,
    },
};
#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use ink_primitives::Key;
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

/// A stash collection.
///
/// Provides O(1) random insertion, deletion and access of its elements.
///
/// # Details
///
/// An `O(1)` amortized table that reuses keys.
///
/// ## Guarantees and non-guarantees:
///
/// 1. `Stash` is deterministic and keys do not depend on the inserted values.
///    This means you can update two stashes in tandem and get the same keys
///    back. This could be useful for, e.g., primary/secondary replication.
/// 2. Keys will always be less than the maximum number of items that have ever
///    been present in the `Stash` at any single point in time. In other words,
///    if you never store more than `n` items in a `Stash`, the stash will only
///    assign keys less than `n`. You can take advantage of this guarantee to
///    truncate the key from a `usize` to some smaller type.
/// 3. Except the guarantees noted above, you can assume nothing about key
///    assignment or iteration order. They can change at any time.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Stash<T> {
    /// Stores densely packed general stash information.
    header: storage::Value<StashHeader>,
    /// The entries of the stash.
    entries: SyncChunk<Entry<T>>,
}

/// Densely stored general information required by a stash.
///
/// # Note
///
/// Separation of these fields into a sub structure has been made
/// for performance reasons so that they all reside in the same
/// storage entity. This allows implementations to perform less reads
/// and writes to the underlying contract storage.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
struct StashHeader {
    /// The latest vacant index.
    next_vacant: u32,
    /// The number of items stored in the stash.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// The maximum length the stash ever had.
    max_len: u32,
}

/// No need to forward flush to fields.
impl ink_core::storage::Flush for StashHeader {}

/// Iterator over the values of a stash.
#[derive(Debug)]
pub struct Values<'a, T> {
    /// The underlying iterator.
    iter: Iter<'a, T>,
}

impl<'a, T> Values<'a, T> {
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a Stash<T>) -> Self {
        Self { iter: stash.iter() }
    }
}

impl<T> Flush for Stash<T>
where
    T: Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.header.flush();
        self.entries.flush();
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasLayout for Stash<T>
where
    T: Metadata + 'static,
{
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(
            Self::meta_type(),
            vec![
                LayoutField::of("header", &self.header),
                LayoutField::of("entries", &self.entries),
            ],
        )
        .into()
    }
}

impl<'a, T> Iterator for Values<'a, T>
where
    T: scale::Codec,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_index, value)| value)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Values<'a, T> where T: scale::Codec {}

impl<'a, T> DoubleEndedIterator for Values<'a, T>
where
    T: scale::Codec,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_index, value)| value)
    }
}

/// Iterator over the entries of a stash.
#[derive(Debug)]
pub struct Iter<'a, T> {
    /// The stash that is iterated over.
    stash: &'a Stash<T>,
    /// The index of the current start item of the iteration.
    begin: u32,
    /// The index of the current end item of the iteration.
    end: u32,
    /// The amount of already yielded items.
    ///
    /// Required to offer an exact `size_hint` implementation.
    /// Also can be used to exit iteration as early as possible.
    yielded: u32,
}

impl<'a, T> Iter<'a, T> {
    /// Creates a new iterator for the given storage stash.
    pub(crate) fn new(stash: &'a Stash<T>) -> Self {
        Self {
            stash,
            begin: 0,
            end: stash.max_len(),
            yielded: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: scale::Codec,
{
    type Item = (u32, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.yielded == self.stash.len() {
            return None
        }
        while self.begin < self.end {
            let cur = self.begin;
            self.begin += 1;
            if let Some(elem) = self.stash.get(cur) {
                self.yielded += 1;
                return Some((cur, elem))
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.stash.len() - self.yielded) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: scale::Codec {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: scale::Codec,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.yielded == self.stash.len() {
            return None
        }
        while self.begin < self.end {
            self.end -= 1;
            if let Some(elem) = self.stash.get(self.end) {
                self.yielded += 1;
                return Some((self.end, elem))
            }
        }
        None
    }
}

/// An entry within a stash collection.
///
/// This represents either an occupied entry with its associated value
/// or a vacant entry pointing to the next vacant entry.
#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
enum Entry<T> {
    /// A vacant entry pointing to the next vacant index.
    Vacant(u32),
    /// An occupied entry containing the value.
    Occupied(T),
}

impl<T> Flush for Entry<T>
where
    T: Flush,
{
    #[inline]
    fn flush(&mut self) {
        match self {
            Entry::Vacant(_) => (),
            Entry::Occupied(occupied) => occupied.flush(),
        }
    }
}

impl<T> Encode for Stash<T> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.header.encode_to(dest);
        self.entries.encode_to(dest);
    }
}

impl<T> Decode for Stash<T> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let header = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        Ok(Self { header, entries })
    }
}

impl<T> AllocateUsing for Stash<T> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            header: storage::Value::allocate_using(alloc),
            entries: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<T> Initialize for Stash<T> {
    type Args = ();

    #[inline(always)]
    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    #[inline]
    fn initialize(&mut self, _args: Self::Args) {
        self.header.set(StashHeader {
            next_vacant: 0,
            len: 0,
            max_len: 0,
        });
    }
}

impl<T> Stash<T> {
    /// Returns an iterator over the references of all entries of the stash.
    ///
    /// # Note
    ///
    /// - It is **not** recommended to iterate over all elements of a storage stash.
    /// - Try to avoid this if possible or iterate only over a minimal subset of
    ///   all elements using e.g. `Iterator::take(n)`.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns an iterator over the references of all values of the stash.
    ///
    /// # Note
    ///
    /// - It is **not** recommended to iterate over all elements of a storage stash.
    /// - Try to avoid this if possible or iterate only over a minimal subset of
    ///   all elements using e.g. `Iterator::take(n)`.
    pub fn values(&self) -> Values<T> {
        Values::new(self)
    }

    /// Returns the underlying key to the cells.
    ///
    /// # Note
    ///
    /// This is a low-level utility getter and should
    /// normally not be required by users.
    pub fn entries_key(&self) -> Key {
        self.entries.cells_key()
    }

    /// Returns the number of elements stored in the stash.
    pub fn len(&self) -> u32 {
        self.header.len
    }

    /// Returns the maximum number of element stored in the
    /// stash at the same time.
    pub fn max_len(&self) -> u32 {
        self.header.max_len
    }

    /// Returns `true` if the stash contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the next vacant index.
    fn next_vacant(&self) -> u32 {
        self.header.next_vacant
    }
}

impl<T> Stash<T>
where
    T: scale::Codec,
{
    /// Returns the element stored at index `n` if any.
    pub fn get(&self, n: u32) -> Option<&T> {
        self.entries.get(n).and_then(|entry| {
            match entry {
                Entry::Occupied(val) => Some(val),
                Entry::Vacant(_) => None,
            }
        })
    }

    /// Put the element into the stash at the next vacant position.
    ///
    /// Returns the stash index that the element was put into.
    pub fn put(&mut self, val: T) -> u32 {
        let current_vacant = self.header.next_vacant;
        debug_assert!(current_vacant <= self.len());
        if current_vacant == self.len() {
            self.entries.set(current_vacant, Entry::Occupied(val));
            self.header.next_vacant = current_vacant + 1;
            self.header.max_len += 1;
        } else {
            let next_vacant = match self
                .entries
                .put(current_vacant, Entry::Occupied(val))
                .expect(
                    "[ink_core::Stash::put] Error: \
                     expected a vacant entry here, but no entry was found",
                ) {
                Entry::Vacant(next_vacant) => next_vacant,
                Entry::Occupied(_) => {
                    unreachable!(
                        "[ink_core::Stash::put] Error: \
                         a next_vacant index can never point to an occupied entry"
                    )
                }
            };
            self.header.next_vacant = next_vacant;
        }
        self.header.len += 1;
        current_vacant
    }

    /// Takes the element stored at index `n`-th if any.
    pub fn take(&mut self, n: u32) -> Option<T> {
        match self.entries.get(n) {
            None | Some(Entry::Vacant(_)) => None,
            Some(Entry::Occupied(_)) => {
                match self
                    .entries
                    .put(n, Entry::Vacant(self.next_vacant()))
                    .expect(
                        "[ink_core::Stash::take] Error: \
                         we already asserted that the entry at `n` exists",
                    ) {
                    Entry::Occupied(val) => {
                        self.header.next_vacant = n;
                        debug_assert!(!self.is_empty());
                        self.header.len -= 1;
                        Some(val)
                    }
                    Entry::Vacant(_) => {
                        unreachable!(
                            "[ink_core::Stash::take] Error: \
                             we already asserted that the entry is occupied"
                        )
                    }
                }
            }
        }
    }
}
