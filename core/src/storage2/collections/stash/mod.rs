// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

mod iter;
mod impls;
mod storage;

#[cfg(test)]
mod tests;

pub use self::iter::{
    Iter,
    IterMut,
};
use crate::storage2::{
    LazyChunk,
    Pack,
    PullForward,
    StorageFootprint,
};
use ink_primitives::Key;

/// An index into the stash.
type Index = u32;

#[derive(Debug)]
pub struct Stash<T> {
    /// The combined and commonly used header data.
    header: Pack<Header>,
    /// The storage entries of the stash.
    entries: LazyChunk<Pack<Entry<T>>>,
}

/// Stores general commonly required information about the storage stash.
#[derive(Debug, scale::Encode, scale::Decode)]
pub struct Header {
    /// The latest vacant index.
    next_vacant: Index,
    /// The number of items stored in the stash.
    ///
    /// # Note
    ///
    /// We cannot simply use the underlying length of the vector
    /// since it would include vacant slots as well.
    len: u32,
    /// The number of entries currently managed by the stash.
    len_entries: u32,
}

/// An entry within the stash.
#[derive(Debug, scale::Encode, scale::Decode)]
pub enum Entry<T> {
    /// A vacant entry that holds the index to the next vacant entry.
    Vacant(Index),
    /// An occupied entry that hold the value.
    Occupied(T),
}

impl<T> Entry<T> {
    /// Returns `true` if the entry is occupied.
    pub fn is_occupied(&self) -> bool {
        if let Entry::Occupied(_) = self {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the entry is vacant.
    pub fn is_vacant(&self) -> bool {
        !self.is_occupied()
    }
}

impl<T> Stash<T> {
    /// Creates a new empty stash.
    pub fn new() -> Self {
        Self {
            header: Pack::new(Header {
                next_vacant: 0,
                len: 0,
                len_entries: 0,
            }),
            entries: LazyChunk::new(),
        }
    }

    /// Returns the number of elements stored in the stash.
    pub fn len(&self) -> u32 {
        self.header.len
    }

    /// Returns `true` if the stash contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of entries currently managed by the storage stash.
    fn len_entries(&self) -> u32 {
        self.header.len_entries
    }

    /// Returns the next vacant index.
    fn next_vacant(&self) -> u32 {
        self.header.next_vacant
    }

    /// Returns the underlying key to the cells.
    ///
    /// # Note
    ///
    /// This is a low-level utility getter and should
    /// normally not be required by users.
    pub fn entries_key(&self) -> Option<&Key> {
        self.entries.key()
    }

    /// Returns an iterator yielding shared references to all elements of the stash.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage stashs.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(self)
    }

    /// Returns an iterator yielding exclusive references to all elements of the stash.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage stashs.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut::new(self)
    }
}

impl<T> Stash<T>
where
    T: scale::Decode + StorageFootprint + PullForward,
{
    /// Returns a shared reference to the element at the given index.
    pub fn get(&self, at: Index) -> Option<&T> {
        self.entries.get(at).and_then(|entry| {
            match Pack::as_inner(entry) {
                Entry::Occupied(val) => Some(val),
                Entry::Vacant(_) => None,
            }
        })
    }

    /// Returns an exclusive reference to the element at the given index.
    pub fn get_mut(&mut self, at: Index) -> Option<&mut T> {
        self.entries.get_mut(at).and_then(|entry| {
            match Pack::as_inner_mut(entry) {
                Entry::Occupied(val) => Some(val),
                Entry::Vacant(_) => None,
            }
        })
    }
}

impl<T> Stash<T>
where
    T: scale::Codec + StorageFootprint + PullForward,
{
    /// Put the element into the stash at the next vacant position.
    ///
    /// Returns the stash index that the element was put into.
    pub fn put(&mut self, new_value: T) -> Index {
        let current_vacant = self.header.next_vacant;
        debug_assert!(current_vacant <= self.len());
        let new_entry = Some(Pack::new(Entry::Occupied(new_value)));
        if current_vacant == self.len() {
            // Push the new element to the end if all entries are occupied.
            self.entries.put(current_vacant, new_entry);
            self.header.next_vacant = current_vacant + 1;
            self.header.len_entries += 1;
        } else {
            // Put the new element to the most recent vacant index if not all entries are occupied.
            let old_entry = self
                .entries
                .put_get(current_vacant, new_entry)
                .expect("a `next_vacant` index must point to an occupied cell");
            let next_vacant = match Pack::into_inner(old_entry) {
                Entry::Vacant(next_vacant) => next_vacant,
                Entry::Occupied(_) => {
                    unreachable!("a `next_vacant` index must point to a vacant entry")
                }
            };
            self.header.next_vacant = next_vacant;
        }
        self.header.len += 1;
        current_vacant
    }

    /// Takes the element stored at the given index if any.
    pub fn take(&mut self, at: Index) -> Option<T> {
        let next_vacant_index = self.next_vacant();
        match self.entries.get_mut(at) {
            None => None,
            Some(packed) => {
                let entry_mut = Pack::as_inner_mut(packed);
                if entry_mut.is_vacant() {
                    // Bail out of the taken entry is already vacant.
                    return None
                }
                // At this point we know that the entry is occupied with a value.
                let new_vacant_entry = Entry::Vacant(next_vacant_index);
                let taken_entry = core::mem::replace(entry_mut, new_vacant_entry);
                match taken_entry {
                    Entry::Occupied(value) => {
                        self.header.next_vacant = at;
                        self.header.len -= 1;
                        Some(value)
                    }
                    Entry::Vacant(_) => {
                        unreachable!("the entry must be occupied at this point")
                }
            }
        }
    }
}
