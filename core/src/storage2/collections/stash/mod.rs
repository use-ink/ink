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

use crate::storage2::{
    LazyChunk,
    Pack,
    PullAt,
    PullForward,
    PushForward,
    StorageFootprint,
};
use ink_primitives::Key;

type Index = u32;

pub struct Stash<T> {
    /// The combined and commonly used header data.
    header: Pack<Header>,
    /// The storage entries of the stash.
    entries: LazyChunk<Pack<Entry<T>>>,
}

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
    /// The maximum length the stash ever had.
    max_len: u32,
}

#[derive(Debug, scale::Encode, scale::Decode)]
pub enum Entry<T> {
    Vacant(Index),
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

impl<T> PullAt for Entry<T>
where
    T: scale::Decode,
{
    fn pull_at(at: Key) -> Self {
        crate::storage2::pull_single_cell(at)
    }
}

impl<T> Stash<T> {
    /// Creates a new empty stash.
    pub fn new() -> Self {
        Self {
            header: Pack::new(Header {
                next_vacant: 0,
                len: 0,
                max_len: 0,
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

    /// Returns the maximum number of element stored in the
    /// stash at the same time.
    pub fn max_len(&self) -> u32 {
        self.header.max_len
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
            self.header.max_len += 1;
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
        // match self.entries.get(at) {
        //     None => return None,
        //     Some(entry) if Pack::as_inner(entry).is_vacant() => {
        //         return None
        //         // if let Entry::Vacant(_) = Pack::as_inner(entry) {}
        //         // match self
        //         //     .entries
        //         //     .put(at, Entry::Vacant(self.next_vacant()))
        //         //     .expect("already asserted that the entry exists")
        //         // {
        //         //     Entry::Occupied(val) => {
        //         //         self.header.next_vacant = n;
        //         //         debug_assert!(!self.is_empty());
        //         //         self.header.len -= 1;
        //         //         Some(val)
        //         //     }
        //         //     Entry::Vacant(_) => {
        //         //         unreachable!("already asserted that the entry is occupied")
        //         //     }
        //         // }
        //     }
        //     _ => (),
        // }
        // self
        //     .entries
        //     .put_get(at, Some(Entry::Vacant(self.next_vacant())))
        todo!()
    }
}
