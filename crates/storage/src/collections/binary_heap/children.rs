// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

//! A `Children` object consists of two elements: a left and right child.

use crate::traits::{
    forward_clear_packed,
    forward_pull_packed,
    forward_push_packed,
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_primitives::Key;

#[cfg(feature = "std")]
use scale_info::TypeInfo;

/// Each `Children` object may contain up to two elements. It is always
/// stored in one single storage cell. This reduces storage access operations
/// of the binary heap algorithm.
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[derive(scale::Encode, scale::Decode, Default, PartialEq, Eq, Debug)]
pub(super) struct Children<T: PackedLayout + Ord> {
    left: Option<T>,
    right: Option<T>,
}

/// The position which a child has in a `Children` object.
#[derive(Copy, Clone, PartialEq, Debug)]
pub(super) enum ChildPosition {
    Left,
    Right,
}

/// Number of elements stored in each `Children` object.
///
/// Note that the first `Children` object (at index `0`) will only ever
/// contain one element (the root element).
pub(super) const CHILDREN_PER_NODE: u32 = 2;

/// Returns the index of the `Children` object in which the `n`-th element of
/// the heap is stored.
pub(super) fn get_children_storage_index(n: u32) -> u32 {
    if n == 0 {
        return 0
    }
    // The first `Children` object only ever contains the root element:
    // `[Some(root), None]`. So when calculating indices we need to account
    // for the items which have been left empty in the first `Children` object.
    let padding = CHILDREN_PER_NODE - 1;
    (n + padding) / CHILDREN_PER_NODE
}

/// Returns the `ChildPosition` of the `n`-th heap element.
///
/// For example, the element `3` is found at the child position `0`
/// (within the `Children` object at storage index `2`).
pub(super) fn get_child_pos(n: u32) -> ChildPosition {
    let storage_index = get_children_storage_index(n);
    match (storage_index, n) {
        (0, 0) => ChildPosition::Left,
        (0, _) => panic!("first children object contains only the root element"),
        (_, _) => {
            let child_pos = (n - 1) % CHILDREN_PER_NODE;
            match child_pos {
                0 => ChildPosition::Left,
                1 => ChildPosition::Right,
                _ => {
                    unreachable!(
                        "CHILDREN_PER_NODE is 2, following the modulo op index must be 0 or 1"
                    )
                }
            }
        }
    }
}

impl<T> Children<T>
where
    T: PackedLayout + Ord,
{
    /// Creates a new `Children` object with a left and right element.
    pub fn new(left: Option<T>, right: Option<T>) -> Self {
        Self { left, right }
    }

    /// Returns the number of existent children in this object.
    pub fn count(&self) -> usize {
        self.left.is_some() as usize + self.right.is_some() as usize
    }

    /// Returns a shared reference to the element at `which`.
    pub fn child(&self, which: ChildPosition) -> &Option<T> {
        match which {
            ChildPosition::Left => &self.left,
            ChildPosition::Right => &self.right,
        }
    }

    /// Returns an exclusive reference to the element at `which`.
    pub fn child_mut(&mut self, which: ChildPosition) -> &mut Option<T> {
        match which {
            ChildPosition::Left => &mut self.left,
            ChildPosition::Right => &mut self.right,
        }
    }
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        CellLayout,
        Layout,
        LayoutKey,
    };

    impl<T> StorageLayout for Children<T>
    where
        T: PackedLayout + Ord + TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Cell(CellLayout::new::<Children<T>>(LayoutKey::from(
                key_ptr.advance_by(1),
            )))
        }
    }
};

impl<T> SpreadLayout for Children<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = false;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        forward_pull_packed::<Self>(ptr)
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        forward_push_packed::<Self>(self, ptr)
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        forward_clear_packed::<Self>(self, ptr)
    }
}

impl<T> PackedLayout for Children<T>
where
    T: PackedLayout + Ord,
{
    fn push_packed(&self, at: &Key) {
        <Option<T> as PackedLayout>::push_packed(&self.left, at);
        <Option<T> as PackedLayout>::push_packed(&self.right, at);
    }

    fn clear_packed(&self, at: &Key) {
        <Option<T> as PackedLayout>::clear_packed(&self.left, at);
        <Option<T> as PackedLayout>::clear_packed(&self.right, at);
    }

    fn pull_packed(&mut self, at: &Key) {
        <Option<T> as PackedLayout>::pull_packed(&mut self.left, at);
        <Option<T> as PackedLayout>::pull_packed(&mut self.right, at);
    }
}

#[test]
fn get_children_storage_index_works() {
    // root is in cell 0
    assert_eq!(get_children_storage_index(0), 0);

    // element 1 + 2 are childrent of element 0 and
    // should be in one cell together
    assert_eq!(get_children_storage_index(1), 1);
    assert_eq!(get_children_storage_index(2), 1);

    // element 3 and 4 should be in one cell
    assert_eq!(get_children_storage_index(3), 2);
    assert_eq!(get_children_storage_index(4), 2);
}

#[test]
fn get_child_pos_works() {
    assert_eq!(get_child_pos(0), ChildPosition::Left);

    assert_eq!(get_child_pos(1), ChildPosition::Left);
    assert_eq!(get_child_pos(2), ChildPosition::Right);

    assert_eq!(get_child_pos(3), ChildPosition::Left);
    assert_eq!(get_child_pos(4), ChildPosition::Right);

    assert_eq!(get_child_pos(5), ChildPosition::Left);
    assert_eq!(get_child_pos(6), ChildPosition::Right);
}
