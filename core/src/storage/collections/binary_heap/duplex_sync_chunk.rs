// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

//! Provides a wrapper around `SyncChunk` which stores a defined amount
//! of values in one cell (instead of the `SyncChunk` behavior of storing
//! one value per cell). The intention is to reduce expensive fetch
//! operations from storage.
//!
//! **NOTE** This wrapper is geared explicitly towards a binary tree
//! structure -- the value which is stored at index `0` (the root) will
//! always be stored in its own group with no other values in it. The
//! intention is to be able to store child nodes paired together
//! in a group, since for query operations you have to access both
//! elements anyways. This allows to skip one expensive read for every
//! accessed pair.
//!
//! For example, for `COUNT = 2` the first group (at index `0`) will
//! contain `[Some(root), None]`. The subsequent group at index `1`
//! will contain `[Some(value from index 1), Some(value from index 2)]`.
//! The getters and setters exposed by this module take care of mapping
//! to the correct group index.

use crate::storage::{
    chunk::SyncChunk,
    Flush,
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

// Number of values stored in each entry of the `SyncChunk`.
// Note that the first group (at index `0`) will only ever
// contain one value.
const COUNT: u32 = 2;

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct Group<T>([Option<T>; COUNT as usize]);

#[derive(Debug, Encode, Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct DuplexSyncChunk<T>(SyncChunk<Group<T>>);

impl<T> Flush for DuplexSyncChunk<T>
where
    SyncChunk<Group<T>>: Flush,
{
    fn flush(&mut self) {
        self.0.flush();
    }
}

#[cfg(feature = "ink-generate-abi")]
impl<T> HasLayout for DuplexSyncChunk<T>
where
    T: Metadata + 'static,
{
    fn layout(&self) -> StorageLayout {
        LayoutStruct::new(
            Self::meta_type(),
            vec![LayoutField::of("sync_chunk", &self.0)],
        ).into()
    }
}

impl<T> DuplexSyncChunk<T>
where
    T: Codec,
{
    pub fn new(chunk: SyncChunk<Group<T>>) -> DuplexSyncChunk<T> {
        DuplexSyncChunk(chunk)
    }

    /// Returns the value of the `n`-th cell if any.
    pub fn get(&self, n: u32) -> Option<&T> {
        let group = get_group_index(n);
        let in_group = get_ingroup_index(n);
        match self.0.get(group).map(|g| g.0[in_group].as_ref()) {
            None => None,
            Some(v) => v,
        }
    }

    /// Returns the value of the `n`-th cell if any.
    pub fn get_group(&self, group: u32) -> Option<&Group<T>> {
        self.0.get(group)
    }

    /// Returns the value of the `n`-th cell if any.
    pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
        let group = get_group_index(n);
        match self.0.get_mut(group).map(|g| {
            let in_group = get_ingroup_index(n);
            g.0[in_group].as_mut()
        }) {
            None => None,
            Some(v) => v,
        }
    }

    /// Takes the value of the `n`-th cell if any.
    pub fn take(&mut self, n: u32) -> Option<T> {
        let group = get_group_index(n);
        match self.0.take(group) {
            None => None,
            Some(existing_group) => {
                let mut existing_group = existing_group.0;
                let in_group = get_ingroup_index(n);

                let taken = existing_group[in_group].take();
                let _ = self.0.put(group, Group(existing_group));
                taken
            }
        }
    }

    /// Replaces the value of the `n`-th cell and returns its old value if any.
    pub fn put(&mut self, n: u32, new_val: T) -> Option<T> {
        let group = get_group_index(n);
        let in_group = get_ingroup_index(n);
        match self.0.get_mut(group) {
            None => {
                let mut new_group: [Option<T>; COUNT as usize] = Default::default();
                new_group[in_group] = Some(new_val);
                let _ = self.0.put(group, Group(new_group));
                None
            }
            Some(existing_group) => existing_group.0[in_group].replace(new_val),
        }
    }
}

/// Returns the group index of the `n`-th cell.
fn get_group_index(n: u32) -> u32 {
    match n {
        0 => 0,
        _ => {
            // the first group only ever contains a single element:
            // the root node (e.g. for `COUNT = 2`, `[Some(root), None]`).
            // so when calculating indices we need to account for the
            // items which have been left empty in the first group.
            let padding = COUNT - 1;
            (n + padding) / COUNT
        }
    }
}

/// Returns the in-group index of the `n`-th cell.
/// This refers to the index which the cell has within a group.
///
/// For example, for `COUNT = 2` the cell `3` is found at in-group
/// index `0` (within the group at index `2`).
fn get_ingroup_index(n: u32) -> usize {
    let group = get_group_index(n);
    match (group, n) {
        (0, 0) => 0,
        (0, _) => panic!("first group contains only root node"),
        (_, _) => ((n - 1) % COUNT) as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_group_index() {
        assert_eq!(get_group_index(0), 0);

        assert_eq!(get_group_index(1), 1);
        assert_eq!(get_group_index(2), 1);

        assert_eq!(get_group_index(3), 2);
        assert_eq!(get_group_index(4), 2);

        assert_eq!(get_group_index(5), 3);
        assert_eq!(get_group_index(6), 3);

        assert_eq!(get_group_index(7), 4);
    }

    #[test]
    fn should_get_ingroup_index() {
        assert_eq!(get_ingroup_index(0), 0);

        assert_eq!(get_ingroup_index(1), 0);
        assert_eq!(get_ingroup_index(2), 1);

        assert_eq!(get_ingroup_index(3), 0);
        assert_eq!(get_ingroup_index(4), 1);

        assert_eq!(get_ingroup_index(5), 0);
        assert_eq!(get_ingroup_index(6), 1);
    }
}
