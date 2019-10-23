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

use crate::storage::{
    chunk::SyncChunk,
    Flush,
};
use scale::{
    Decode,
    Encode,
};

// Number of values stored in each entry of the `SyncChunk`.
const COUNT: u32 = 3;

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub struct Group<T> ([Option<T>; COUNT as usize]);

#[derive(Debug, Encode, Decode)]
pub struct DuplexSyncChunk<T> (SyncChunk<Group<T>>);

impl<T> Flush for DuplexSyncChunk<T>
where
    T: Flush,
    SyncChunk<Group<T>>: Flush,
{
    fn flush(&mut self) {
        self.0.flush();
    }
}

impl<T> DuplexSyncChunk<T>
where
    T: scale::Encode + scale::Decode + Copy + Clone,
{
    pub fn new(chunk: SyncChunk<Group<T>>) -> DuplexSyncChunk<T> {
        DuplexSyncChunk(chunk)
    }

    /// Returns the value of the `n`-th cell if any.
    pub fn get(&self, n: u32) -> Option<&T> {
        let group = n / COUNT;
        let in_group = n % COUNT;
        match self.0.get(group).map(|g| {
            g.0[in_group as usize].as_ref()
        }) {
            None => None,
            Some(v) => v,
        }
    }

    /// Returns the value of the `n`-th cell if any.
    pub fn get_mut(&mut self, n: u32) -> Option<&mut T> {
        let group = n / COUNT;
        match self.0.get_mut(group).map(|g| {
            let in_group = (n % COUNT) as usize;
            g.0[in_group].as_mut()
        }) {
            None => None,
            Some(v) => v,
        }
    }

    /// Takes the value of the `n`-th cell if any.
    pub fn take(&mut self, n: u32) -> Option<T> {
        let group = n / COUNT;
        match self.0.take(group) {
            None => None,
            Some(existing_group) => {
                let mut existing_group = existing_group.0;
                let in_group = (n % COUNT) as usize;

                let taken = existing_group[in_group];
                existing_group[in_group] = None;
                let _ = self.0.put(group, Group(existing_group));
                taken
            },
        }
    }

    /// Replaces the value of the `n`-th cell and returns its old value if any.
    pub fn put(&mut self, n: u32, new_val: T) -> Option<T> {
        let group = n / COUNT;
        let in_group = (n % COUNT) as usize;
        match self.0.get_mut(group) {
            None => {
                let mut new_group: [Option<T>; COUNT as usize] = Default::default();
                new_group[in_group] = Some(new_val);
                let _ = self.0.put(group, Group(new_group));
                None
            },
            Some(existing_group) => {
                let prior = existing_group.0[in_group];
                existing_group.0[in_group] = Some(new_val);
                prior
            },
        }
    }
}
