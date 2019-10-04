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

//! The storage emulates the chain's storage as well as the unique
//! storage of the executed contract.
//!
//! It is a map from `Key` (32-bytes) to a generic `Vec<u8>`.
//! This is pretty close to the actual on-chain storage in Substrate.

use crate::{
    memory::collections::btree_map::{
        self,
        BTreeMap,
    },
    storage::Key,
};
use core::cell::Cell;

/// An entry in the storage of the test environment.
///
/// # Note
///
/// Additionally to its data it also stores the total
/// number of reads and writes done to this entry.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The actual data that is stored in this storage entry.
    data: Vec<u8>,
    /// The number of reads to this storage entry.
    reads: Cell<usize>,
    /// The number of writes to this storage entry.
    writes: usize,
}

impl Entry {
    /// Creates a new storage entry for the given data.
    pub fn new<V>(data: V) -> Self
    where
        V: Into<Vec<u8>>,
    {
        Self {
            data: data.into(),
            reads: Cell::new(0),
            writes: 0,
        }
    }

    /// Increases the read counter by one.
    fn inc_reads(&self) {
        self.reads.set(self.reads.get() + 1);
    }

    /// Increases the write counter by one.
    fn inc_writes(&mut self) {
        self.writes += 1;
    }

    /// Returns the number of reads for this storage entry.
    pub fn reads(&self) -> usize {
        self.reads.get()
    }

    /// Returns the number of writes to this storage entry.
    pub fn writes(&self) -> usize {
        self.writes
    }

    /// Returns the data stored in this storage entry.
    ///
    /// # Note
    ///
    /// Also bumps the read counter.
    pub fn data(&self) -> &[u8] {
        self.inc_reads();
        &self.data
    }

    /// Overwrites the data of this entry.
    ///
    /// # Note
    ///
    /// Also bumps the write counter.
    pub fn overwrite<V>(&mut self, new_data: V)
    where
        V: Into<Vec<u8>>,
    {
        self.inc_writes();
        self.data = new_data.into();
    }
}

/// The storage of a smart contract.
///
/// This is a mapping from `Key` to entries
/// that just store the raw byte representation
/// of whatever they are storing.
///
/// Also every entry and the storage itself are
/// storing counts of read and write accesses for
/// further introspection of storage access.
#[derive(Debug, Clone)]
pub struct Storage {
    /// All storage entries.
    entries: BTreeMap<Key, Entry>,
    /// The total number of reads from the storage.
    total_reads: Cell<usize>,
    /// The total number of writes to the storage.
    total_writes: usize,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            total_reads: Cell::new(0),
            total_writes: 0,
        }
    }
}

impl Storage {
    /// Writes the given value to the storage under the key.
    ///
    /// # Note
    ///
    /// Overwrites if there was already a value stored under the key.
    pub fn write<V>(&mut self, key: Key, value: V)
    where
        V: Into<Vec<u8>>,
    {
        self.inc_total_writes();
        match self.entries.entry(key) {
            btree_map::Entry::Occupied(mut occupied) => {
                occupied.get_mut().overwrite(value);
            }
            btree_map::Entry::Vacant(vacant) => {
                vacant.insert(Entry::new(value));
            }
        }
    }

    /// Clears the entry under the given key.
    pub fn clear(&mut self, key: Key) {
        self.inc_total_writes();
        self.entries.remove(&key);
    }

    /// Reads the data from the storage.
    ///
    /// # Note
    ///
    /// This lso resets the read & write counters for
    /// the same entry in the current implementation.
    pub fn read(&self, key: Key) -> Option<&Entry> {
        self.inc_total_reads();
        self.entries.get(&key)
    }

    /// Increases the total reads counter by one.
    fn inc_total_reads(&self) {
        self.total_reads.set(self.total_reads.get() + 1);
    }

    /// Increases the total writes counter by one.
    fn inc_total_writes(&mut self) {
        self.total_writes += 1;
    }

    /// Returns the number of reads for this storage entry.
    pub fn reads(&self) -> usize {
        self.total_reads.get()
    }

    /// Returns the number of writes to this storage entry.
    pub fn writes(&self) -> usize {
        self.total_writes
    }
}
