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

//! Provides an interface around the vector used to store elements of the
//! [`BinaryHeap`](`super::BinaryHeap`) in storage. This is necessary since
//! we don't just store each element in it's own storage cell, but rather
//! optimize storage access by putting children together in one storage cell.

use crate::storage2::{
    collections::binary_heap::{
        children,
        children::{
            ChildPosition,
            Children,
        },
        Iter,
        IterMut,
        StorageVec,
    },
    traits::{
        KeyPtr,
        PackedLayout,
        SpreadLayout,
    },
    Lazy,
};

/// Provides an interface for accessing elements in the `BinaryHeap`.
///
/// Elements are stored in a vector of `Children` objects, whereby
/// each object contains two elements.
/// When operating on element indices the index needs to be transposed
/// to the `Children` object in which the element is stored.
#[derive(Default, PartialEq, Eq, Debug)]
pub struct Elements<T>
where
    T: PackedLayout + Ord,
{
    /// The number of elements stored in the heap.
    /// We cannot use the length of the storage vector, since each entry (i.e. each
    /// `Children` object) in the vector contains two child elements (except the root
    /// element which occupies a `Children` object on its own.
    len: Lazy<u32>,
    /// The underlying storage vec containing the `Children`.
    children: StorageVec<Children<T>>,
}

impl<T> Elements<T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new empty storage heap.
    pub fn new() -> Self {
        Self {
            len: Lazy::new(0),
            children: StorageVec::new(),
        }
    }

    /// Returns the number of elements in the heap, also referred to as its 'length'.
    pub fn len(&self) -> u32 {
        *self.len
    }

    /// Returns the amount of `Children` objects stored in the vector.
    #[allow(dead_code)]
    #[cfg(all(test, feature = "ink-fuzz-tests"))]
    pub fn children_count(&self) -> u32 {
        self.children.len()
    }

    /// Returns `true` if the heap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a shared reference to the indexed element.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get(&self, index: u32) -> Option<&T> {
        let storage_index = children::get_children_storage_index(index);
        let i = self.within_bounds(storage_index)?;
        let children = self.children.get(i)?;
        let child_pos = children::get_child_pos(index);
        children.child(child_pos)
    }

    /// Returns an exclusive reference to the indexed element.
    /// The element in a `Children` is an `Option<T>`.
    ///
    /// Returns `None` if `index` is out of bounds.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut Option<T>> {
        let storage_index = children::get_children_storage_index(index);
        self.within_bounds(storage_index)?;
        self.children.get_mut(storage_index).map(|children| {
            let child_pos = children::get_child_pos(index);
            children.child_mut(child_pos)
        })
    }

    /// Swaps the elements at the given indices.
    ///
    /// # Panics
    ///
    /// If one or both indices are out of bounds.
    pub fn swap(&mut self, a: u32, b: u32) {
        if a == b {
            return
        }
        assert!(a < self.len(), "a is out of bounds");
        assert!(b < self.len(), "b is out of bounds");

        let old_a = self.get_mut(a).expect("index a must exist").take();

        let old_b = {
            let b_opt = self.get_mut(b).expect("index b must exist");
            let old_b = b_opt.take();
            *b_opt = old_a;
            old_b
        };

        *self.get_mut(a).expect("index a must exist") = old_b;
    }

    /// Removes the indexed element from the vector and returns it.
    ///
    /// The last element of the vector is put into the indexed slot.
    /// Returns `None` and does not mutate the vector if the index is out of bounds.
    ///
    /// # Note
    ///
    /// This operation does not preserve ordering but is constant time.
    pub fn swap_remove(&mut self, n: u32) -> Option<T> {
        if self.is_empty() {
            return None
        }
        self.swap(n, self.len() - 1);
        self.pop()
    }

    /// Returns an iterator yielding shared references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter(&self) -> Iter<T> {
        Iter::new(&self.children)
    }

    /// Returns an iterator yielding exclusive references to all elements of the vector.
    ///
    /// # Note
    ///
    /// Avoid unbounded iteration over big storage vectors.
    /// Prefer using methods like `Iterator::take` in order to limit the number
    /// of yielded elements.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        // self.elems.iter_mut()
        IterMut::new(&mut self.children)
    }

    /// Returns a shared reference to the first element if any.
    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            return None
        }
        self.get(0)
    }

    /// Returns an exclusive reference to the first element if any.
    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None
        }
        self.get_mut(0)?.as_mut()
    }

    /// Removes all elements from this vector.
    ///
    /// # Note
    ///
    /// Use this method to clear the vector instead of e.g. iterative `pop()`.
    /// This method performs significantly better and does not actually read
    /// any of the elements (whereas `pop()` does).
    pub fn clear(&mut self) {
        if self.is_empty() {
            return
        }
        self.children.clear();
        self.len = Lazy::new(0);
    }

    /// Appends an element to the back of the vector.
    pub fn push(&mut self, value: T) {
        assert!(
            self.len() < core::u32::MAX,
            "cannot push more elements into the storage vector"
        );
        let last_index = self.len();
        *self.len += 1;
        self.push_to(last_index, Some(value));
    }

    fn push_to(&mut self, index: u32, value: Option<T>) {
        let children_index = children::get_children_storage_index(index);
        match self.children.get_mut(children_index) {
            Some(children) => {
                let child_pos = children::get_child_pos(index);
                *children.child_mut(child_pos) = value;
            }
            None => {
                self.children.push(Children::new(value, None));
                debug_assert!(
                    self.children.get(children_index).is_some(),
                    "the new children were not placed at children_index!"
                );
            }
        };
    }

    /// Pops the last element from the vector and returns it.
    //
    /// Returns `None` if the vector is empty.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None
        }
        let last_index = self.len() - 1;
        *self.len = last_index;

        let children_index = children::get_children_storage_index(last_index);
        let old = self.children.get_mut(children_index);
        match old {
            Some(children) => {
                let child_pos = children::get_child_pos(last_index);
                let popped_val = children.child_mut(child_pos).take();

                // if both children are non-existent the entire children object can be removed
                if !children.exists(ChildPosition::Left)
                    && !children.exists(ChildPosition::Right)
                {
                    self.children.pop();
                }

                popped_val
            }
            None => {
                unreachable!("vec must contain children at children_index of last_index")
            }
        }
    }

    /// Returns the index if it is within bounds or `None` otherwise.
    fn within_bounds(&self, index: u32) -> Option<u32> {
        if index < self.len() {
            return Some(index)
        }
        None
    }
}

impl<T> SpreadLayout for Elements<T>
where
    T: SpreadLayout + Ord + PackedLayout,
{
    const FOOTPRINT: u64 = 1 + <StorageVec<Children<T>> as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let len = SpreadLayout::pull_spread(ptr);
        let elems = SpreadLayout::pull_spread(ptr);
        Self {
            len,
            children: elems,
        }
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.len, ptr);
        SpreadLayout::push_spread(&self.children, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.len, ptr);
        SpreadLayout::clear_spread(&self.children, ptr);
    }
}
