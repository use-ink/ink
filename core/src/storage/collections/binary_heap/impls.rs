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

use core::cmp::{
    Ord,
    Ordering,
};

#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use scale::{
    Codec,
    Decode,
    Encode,
};
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

use super::duplex_sync_chunk::DuplexSyncChunk;
use crate::storage::{
    self,
    alloc::{
        Allocate,
        AllocateUsing,
        Initialize,
    },
    chunk::SyncChunk,
    Flush,
};

/// We implement a binary tree.
pub const CHILDREN: u32 = 2;

/// A binary heap collection.
/// The heap depends on `Ord` and is a max-heap by default. In order to
/// make it a min-heap implement the `Ord` trait explicitly on the type
/// which is stored in the heap.
///
/// Provides `O(log(n))` push and pop operations.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct BinaryHeap<T> {
    /// The number of nodes stored in the heap.
    len: storage::Value<u32>,
    /// The nodes of the heap.
    entries: DuplexSyncChunk<T>,
}

/// Iterator over the values of a heap.
#[derive(Debug)]
pub struct Values<'a, T> {
    /// The underlying iterator.
    iter: Iter<'a, T>,
}

impl<'a, T> Values<'a, T> {
    /// Creates a new iterator for the given storage heap.
    pub(crate) fn new(heap: &'a BinaryHeap<T>) -> Self
    where
        T: scale::Codec + Ord,
    {
        Self { iter: heap.iter() }
    }
}

impl<T> Flush for BinaryHeap<T>
where
    T: Encode + Flush,
    DuplexSyncChunk<T>: Flush,
{
    fn flush(&mut self) {
        self.len.flush();
        self.entries.flush();
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasLayout for BinaryHeap<T>
where
    T: Metadata + 'static,
{
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(
            Self::meta_type(),
            vec![
                LayoutField::of("len", &self.len),
                LayoutField::of("entries", &self.entries),
            ],
        )
        .into()
    }
}

impl<'a, T> Iterator for Values<'a, T>
where
    T: Codec + Ord,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(_index, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> ExactSizeIterator for Values<'a, T> where T: Codec + Ord {}

impl<'a, T> DoubleEndedIterator for Values<'a, T>
where
    T: Codec + Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|(_index, value)| value)
    }
}

/// Iterator over the elements of a heap. The iteration is not
/// guaranteed to be ordered, it is arbitrary!
#[derive(Debug)]
pub struct Iter<'a, T> {
    /// The heap that is iterated over.
    heap: &'a BinaryHeap<T>,
    /// The index of the current start node of the iteration.
    begin: u32,
    /// The index of the current end node of the iteration.
    end: u32,
    /// The amount of already yielded nodes.
    ///
    /// Required to offer an exact `size_hint` implementation.
    /// Also can be used to exit iteration as early as possible.
    yielded: u32,
}

impl<'a, T> Iter<'a, T> {
    /// Creates a new iterator for the given storage heap.
    pub(crate) fn new(heap: &'a BinaryHeap<T>) -> Self
    where
        T: Codec + Ord,
    {
        Self {
            heap,
            begin: 0,
            end: heap.len(),
            yielded: 0,
        }
    }
}

impl<'a, T> Iterator for Iter<'a, T>
where
    T: Codec + Ord,
{
    type Item = (u32, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.yielded == self.heap.len() {
            return None
        }
        while self.begin < self.end {
            let cur = self.begin;
            self.begin += 1;
            if let Some(elem) = self.heap.get(cur) {
                self.yielded += 1;
                return Some((cur, elem))
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.heap.len() - self.yielded) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> where T: Codec + Ord {}

impl<'a, T> DoubleEndedIterator for Iter<'a, T>
where
    T: Codec + Ord,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        debug_assert!(self.begin <= self.end);
        if self.yielded == self.heap.len() {
            return None
        }
        while self.begin < self.end {
            self.end -= 1;
            if let Some(elem) = self.heap.get(self.end) {
                self.yielded += 1;
                return Some((self.end, elem))
            }
        }
        None
    }
}

impl<T> Encode for BinaryHeap<T> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.len.encode_to(dest);
        self.entries.encode_to(dest);
    }
}

impl<T> Decode for BinaryHeap<T>
where
    T: Codec,
{
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let len = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        Ok(Self {
            len,
            entries: DuplexSyncChunk::new(entries),
        })
    }
}

impl<T> AllocateUsing for BinaryHeap<T>
where
    T: Codec,
{
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            len: storage::Value::allocate_using(alloc),
            entries: DuplexSyncChunk::new(SyncChunk::allocate_using(alloc)),
        }
    }
}

impl<T> Initialize for BinaryHeap<T> {
    type Args = ();

    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    fn initialize(&mut self, _: Self::Args) {
        self.len.set(0);
    }
}

impl<T> BinaryHeap<T>
where
    T: Codec + Ord,
{
    /// Returns the element stored at index `n` if any.
    pub fn len(&self) -> u32 {
        *self.len.get()
    }

    /// Returns `true` if the heap is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the first node if not empty.
    pub fn peek(&self) -> Option<&T> {
        self.entries.get(0)
    }

    /// Mutates the first node if not empty and returns a reference to the result.
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.entries.get_mut(0)
    }

    /// If the heap is not empty the first node is returned and removed.
    ///
    /// Complexity is `O(log(n))`.
    pub fn pop(&mut self) -> Option<T> {
        let len = self.len();
        if len == 0 {
            return None
        }

        let tmp = Some(self.entries.take(0).expect("failed fetching root"));
        if len == 1 {
            self.len.set(len - 1);
            return tmp
        }

        self.relocate(len - 1, 0);

        self.len.set(len - 1);
        self.repair_top();
        tmp
    }

    /// Move the top of the heap to its correct place within the heap, so that
    /// sort order is maintained.
    fn repair_top(&mut self) {
        let mut top_index = 0;
        let top_value = self
            .entries
            .take(top_index)
            .expect("failed taking top element from heap");
        let mut succ_index = self.find_successor(top_index);
        while succ_index < self.len() && {
            let succ_value = self
                .entries
                .get(succ_index)
                .expect("failed retrieving successor");
            top_value < *succ_value
        } {
            self.relocate(succ_index, top_index);
            top_index = succ_index;
            succ_index = self.find_successor(succ_index);
        }
        let _ = self.entries.put(top_index, top_value);
    }

    /// Returns the index of the child node with the largest value.
    ///
    /// The `index` parameter refers to the parent node.
    fn find_successor(&mut self, index: u32) -> u32 {
        let left_index = index * CHILDREN + 1;
        let right_index = index * CHILDREN + 2;
        if right_index >= self.len() {
            return left_index
        }

        let left = self
            .entries
            .get(left_index)
            .expect("failed getting left value");
        let right = self
            .entries
            .get(right_index)
            .expect("failed getting right value");
        match left.cmp(right) {
            Ordering::Less => right_index,
            Ordering::Equal => right_index,
            Ordering::Greater => left_index,
        }
    }

    /// Pushes an item onto the heap.
    ///
    /// Panics in case the heap already contains `u32::max` nodes.
    /// Complexity is `O(log(n))`.
    pub fn push(&mut self, val: T) {
        let len = self.len();
        if len == u32::max_value() {
            panic!(
                "[ink_core::Heap::push] Error: \
                 cannot push more elements than `u32::Max`"
            )
        }

        if len == 0 {
            let _ = self.entries.put(0, val);
            self.len.set(len + 1);
            return
        }

        // Relocate until the item is greater than its parent value.
        let mut index = len;
        let mut parent_index = self.parent_index(index);
        while index != 0 && {
            let parent_value = self
                .entries
                .get(parent_index)
                .expect("failed getting parent value");
            val > *parent_value
        } {
            self.relocate(parent_index, index);

            index = parent_index;
            if index > 0 {
                parent_index = self.parent_index(index);
            }
        }
        self.len.set(self.len() + 1);
        let _ = self.entries.put(index, val);
    }

    /// Returns an iterator over the references of all nodes of the heap.
    /// The order is arbitrary!
    ///
    /// # Note
    ///
    /// - The iteration is not guaranteed to be ordered!
    /// - It is **not** recommended to iterate over all elements of a storage heap.
    /// - Try to avoid this if possible or iterate only over a minimal subset of
    ///   all elements using e.g. `Iterator::take(n)`.
    pub fn values(&self) -> Values<T> {
        Values::new(self)
    }

    /// Returns the node at index `n`.
    fn get(&self, n: u32) -> Option<&T> {
        self.entries.get(n)
    }

    /// Relocate the node at index `from` to `to`.
    /// Overwrites the node at `to`.
    fn relocate(&mut self, from: u32, to: u32) {
        let entry = self.entries.take(from).expect("failed relocating node");
        let _ = self.entries.put(to, entry);
    }

    /// Returns the parent index of the node at `n`.
    fn parent_index(&self, n: u32) -> u32 {
        (n - 1) / CHILDREN
    }

    /// Returns an iterator over all nodes of the heap.
    /// The order is arbitrary!
    ///
    /// # Note
    ///
    /// - The iteration is not guaranteed to be ordered!
    /// - It is **not** recommended to iterate over all elements of a storage heap.
    /// - Try to avoid this if possible or iterate only over a minimal subset of
    ///   all elements using e.g. `Iterator::take(n)`.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }
}

impl<T> Extend<T> for BinaryHeap<T>
where
    T: Codec + Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for i in iter {
            self.push(i)
        }
    }
}

impl<'a, T> Extend<&'a T> for BinaryHeap<T>
where
    T: Codec + Ord + Copy + 'a,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = &'a T>,
    {
        self.extend(iter.into_iter().copied())
    }
}
