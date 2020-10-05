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

//! A `Group` consists of two elements of the [`BinaryHeap`](`super::BinaryHeap`).
//! These are the children of an element. The root element is an exception, as
//! it occupies a `Group` on its own.

use crate::storage2::traits::{
    KeyPtr,
    PackedLayout,
    SpreadLayout,
};
use ink_primitives::Key;
use scale_info::TypeInfo;

/// Each group may contain up to two elements. Each group is always stored
/// in one single storage cell; this reduced read operations of the
/// binary heap algorithm.
#[cfg_attr(feature = "std", derive(TypeInfo))]
#[derive(scale::Encode, scale::Decode, Default, PartialEq, Eq, Debug)]
pub struct Group<T: PackedLayout + Ord>(pub Option<T>, pub Option<T>);

/// The position which an element has in a `Group`.
#[derive(PartialEq, Debug)]
pub enum Ingroup {
    Left,
    Right,
}

/// Number of element stored in each group.
/// Note that the first group (at index `0`) will only ever
/// contain one element (the root element).
const COUNT: u32 = 2;

/// Returns the index of the group in which the `n`-th element is stored.
pub(crate) fn get_group_index(n: u32) -> u32 {
    match n {
        0 => 0,
        _ => {
            // The first group only ever contains the root element:
            // `[Some(root), None]`. So when calculating indices we
            // need to account for the items which have been left
            // empty in the first group.
            let padding = COUNT - 1;
            (n + padding) / COUNT
        }
    }
}

/// Returns the in-group index of the `n`-th element.
/// This refers to the index which the element has within a group.
///
/// For example, the element `3` is found at in-group index `0`
/// (within the group at index `2`).
pub(crate) fn get_ingroup_index(n: u32) -> Ingroup {
    let group = get_group_index(n);
    match (group, n) {
        (0, 0) => Ingroup::Left,
        (0, _) => panic!("first group contains only the root element"),
        (_, _) => {
            let ingroup_index = (n - 1) % COUNT;
            match ingroup_index {
                0 => Ingroup::Left,
                1 => Ingroup::Right,
                _ => {
                    unreachable!(
                        "COUNT is 2, following the modulo op index must be 0 or 1"
                    )
                }
            }
        }
    }
}

impl<T> Group<T>
where
    T: PackedLayout + Ord,
{
    /// Returns a shared reference to the element at `index`.
    pub fn as_ref(&self, index: u32) -> Option<&T> {
        let ingroup_index = get_ingroup_index(index);
        match ingroup_index {
            Ingroup::Left => self.0.as_ref(),
            Ingroup::Right => self.1.as_ref(),
        }
    }

    /// Returns an exclusive reference to the element at `index`.
    pub fn get_mut(&mut self, index: u32) -> &mut Option<T> {
        let ingroup_index = get_ingroup_index(index);
        match ingroup_index {
            Ingroup::Left => &mut self.0,
            Ingroup::Right => &mut self.1,
        }
    }
}

#[cfg(feature = "std")]
const _: () = {
    use crate::storage2::traits::StorageLayout;
    use ink_metadata::layout2::{
        CellLayout,
        FieldLayout,
        Layout,
        LayoutKey,
        StructLayout,
    };

    impl<T> StorageLayout for Group<T>
    where
        T: PackedLayout + Ord + TypeInfo + 'static,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            StructLayout::new(vec![
                FieldLayout::new(
                    None,
                    CellLayout::new::<i32>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
                FieldLayout::new(
                    None,
                    CellLayout::new::<i64>(LayoutKey::from(key_ptr.advance_by(1))),
                ),
            ])
            .into()
        }
    }
};

impl<T> SpreadLayout for Group<T>
where
    T: PackedLayout + Ord,
{
    const FOOTPRINT: u64 = 2 * <T as SpreadLayout>::FOOTPRINT;

    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        Self(
            SpreadLayout::pull_spread(ptr),
            SpreadLayout::pull_spread(ptr),
        )
    }

    fn push_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::push_spread(&self.0, ptr);
        SpreadLayout::push_spread(&self.1, ptr);
    }

    fn clear_spread(&self, ptr: &mut KeyPtr) {
        SpreadLayout::clear_spread(&self.0, ptr);
        SpreadLayout::clear_spread(&self.1, ptr);
    }
}

impl<T> PackedLayout for Group<T>
where
    T: PackedLayout + Ord,
{
    fn push_packed(&self, at: &Key) {
        <Option<T> as PackedLayout>::push_packed(&self.0, at);
        <Option<T> as PackedLayout>::push_packed(&self.1, at);
    }

    fn clear_packed(&self, at: &Key) {
        <Option<T> as PackedLayout>::clear_packed(&self.0, at);
        <Option<T> as PackedLayout>::clear_packed(&self.1, at);
    }

    fn pull_packed(&mut self, at: &Key) {
        <Option<T> as PackedLayout>::pull_packed(&mut self.0, at);
        <Option<T> as PackedLayout>::pull_packed(&mut self.1, at);
    }
}

#[test]
fn get_group_index_works() {
    // root is in cell 0
    assert_eq!(get_group_index(0), 0);

    // element 1 + 2 are childrent of element 0 and
    // should be in one cell together
    assert_eq!(get_group_index(1), 1);
    assert_eq!(get_group_index(2), 1);

    // element 3 and 4 should be in one cell
    assert_eq!(get_group_index(3), 2);
    assert_eq!(get_group_index(4), 2);
}

#[test]
fn should_get_ingroup_index() {
    assert_eq!(get_ingroup_index(0), Ingroup::Left);

    assert_eq!(get_ingroup_index(1), Ingroup::Left);
    assert_eq!(get_ingroup_index(2), Ingroup::Right);

    assert_eq!(get_ingroup_index(3), Ingroup::Left);
    assert_eq!(get_ingroup_index(4), Ingroup::Right);

    assert_eq!(get_ingroup_index(5), Ingroup::Left);
    assert_eq!(get_ingroup_index(6), Ingroup::Right);
}
