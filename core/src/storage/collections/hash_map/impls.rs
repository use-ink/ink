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

// We allow having no generalization for hasher of the hashmap implementation.
// This might change in future versions of the pDSL.
#![allow(clippy::implicit_hasher)]

use core::{
    borrow::Borrow,
    hash::Hash,
};

#[cfg(feature = "ink-generate-abi")]
use ink_abi::{
    HasLayout,
    LayoutField,
    LayoutStruct,
    StorageLayout,
};
use ink_primitives::hash;
#[cfg(feature = "ink-generate-abi")]
use type_metadata::Metadata;

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

/// Mapping stored in the contract storage.
///
/// # Note
///
/// This performs a quadratic probing on the next 2^32 slots
/// following its initial key. So it can store up to 2^32 elements in total.
///
/// Instead of storing element values (`V`) directly, it stores
/// storage map entries of `(K, V)` instead. This allows to represent
/// the storage that is associated to the storage map to be in three
/// different states.
///
/// 1. Occupied slot with key and value.
/// 2. Removed slot that was occupied before.
/// 3. Empty slot when there never was an insertion for this storage slot.
///
/// This distinction is important for the quadratic map probing.
#[derive(Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct HashMap<K, V> {
    /// The storage key to the length of this storage map.
    len: storage::Value<u32>,
    /// The first half of the entry buffer is equal to the key,
    /// the second half will be replaced with the respective
    /// hash of any given key upon usage.
    ///
    /// Afterwards this value is hashed again and used as key
    /// into the contract storage.
    entries: SyncChunk<Entry<K, V>>,
}

#[cfg(feature = "ink-generate-abi")]
impl<K, V> HasLayout for HashMap<K, V>
where
    K: Metadata + 'static,
    V: Metadata + 'static,
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

/// An entry of a storage map.
///
/// This can either store the entries key and value
/// or represent an entry that was removed after it
/// has been occupied with key and value.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub enum Entry<K, V> {
    /// An occupied slot with a key and a value.
    Occupied(OccupiedEntry<K, V>),
    /// A removed slot that was occupied before.
    Removed,
}

impl<K, V> Flush for Entry<K, V>
where
    K: scale::Encode + Flush,
    V: scale::Encode + Flush,
{
    fn flush(&mut self) {
        match self {
            Entry::Occupied(occupied) => occupied.flush(),
            Entry::Removed => (),
        }
    }
}

/// An occupied entry of a storage map.
#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
pub struct OccupiedEntry<K, V> {
    /// The entry's key.
    key: K,
    /// The entry's value.
    val: V,
}

impl<K, V> Flush for OccupiedEntry<K, V>
where
    K: scale::Encode + Flush,
    V: scale::Encode + Flush,
{
    fn flush(&mut self) {
        self.key.flush();
        self.val.flush();
    }
}

impl<K, V> Flush for HashMap<K, V>
where
    K: scale::Encode + Flush,
    V: scale::Encode + Flush,
{
    #[inline]
    fn flush(&mut self) {
        self.len.flush();
        self.entries.flush();
    }
}

impl<K, V> scale::Encode for HashMap<K, V> {
    fn encode_to<W: scale::Output>(&self, dest: &mut W) {
        self.len.encode_to(dest);
        self.entries.encode_to(dest);
    }
}

impl<K, V> scale::Decode for HashMap<K, V> {
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        let len = storage::Value::decode(input)?;
        let entries = SyncChunk::decode(input)?;
        Ok(Self { len, entries })
    }
}

impl<K, V> AllocateUsing for HashMap<K, V> {
    #[inline]
    unsafe fn allocate_using<A>(alloc: &mut A) -> Self
    where
        A: Allocate,
    {
        Self {
            len: storage::Value::allocate_using(alloc),
            entries: SyncChunk::allocate_using(alloc),
        }
    }
}

impl<K, V> Initialize for HashMap<K, V> {
    type Args = ();

    #[inline(always)]
    fn default_value() -> Option<Self::Args> {
        Some(())
    }

    #[inline]
    fn initialize(&mut self, _args: Self::Args) {
        self.len.set(0);
    }
}

impl<K, V> Extend<(K, V)> for HashMap<K, V>
where
    K: scale::Codec + Hash + Eq,
    V: scale::Codec,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<'a, K, V> Extend<(&'a K, &'a V)> for HashMap<K, V>
where
    K: scale::Codec + Hash + Eq + Copy,
    V: scale::Codec + Copy,
{
    fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
    }
}

impl<'a, K, V> Extend<&'a (K, V)> for HashMap<K, V>
where
    K: scale::Codec + Hash + Eq + Copy + 'a,
    V: scale::Codec + Copy + 'a,
{
    fn extend<I: IntoIterator<Item = &'a (K, V)>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

impl<K, V> HashMap<K, V> {
    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> u32 {
        *self.len.get()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Converts the given slice into an array with fixed size of 4.
///
/// Returns `None` if the slice's length is not 4.
fn slice_as_array4<T>(bytes: &[T]) -> Option<[T; 4]>
where
    T: Default + Copy,
{
    if bytes.len() != 4 {
        return None
    }
    let mut array = [T::default(); 4];
    array[..4].clone_from_slice(&bytes[..4]);
    Some(array)
}

impl<K, V> HashMap<K, V>
where
    K: scale::Codec + Hash + Eq,
    V: scale::Codec,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated,
    /// and the old value is returned.
    /// The key is not updated, though;
    /// this matters for types that can be == without being identical.
    /// See the module-level documentation for more.
    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        match self.probe_inserting(&key) {
            Some(ProbeSlot::Occupied(probe_index)) => {
                // Keys match, values might not.
                self.entries
                    .put(probe_index, Entry::Occupied(OccupiedEntry { key, val }))
                    .and_then(|old| {
                        match old {
                            Entry::Occupied(OccupiedEntry { val, .. }) => Some(val),
                            Entry::Removed => None,
                        }
                    })
            }
            Some(ProbeSlot::Vacant(probe_index)) => {
                // We can insert into this slot.
                self.len.set(self.len() + 1);
                self.entries
                    .set(probe_index, Entry::Occupied(OccupiedEntry { key, val }));
                None
            }
            None => {
                panic!("[ink_core::HashMap::insert] Error: failed finding a valid entry")
            }
        }
    }
}

/// The result of a slot probe.
enum ProbeSlot {
    /// The probed slot is empty or removed.
    Vacant(u32),
    /// The probed slot is occupied.
    Occupied(u32),
}

impl ProbeSlot {
    /// Returns the index of the probe slot.
    fn index(&self) -> u32 {
        match self {
            ProbeSlot::Vacant(index) | ProbeSlot::Occupied(index) => *index,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: scale::Codec,
    V: scale::Codec,
{
    /// The maximum amount of probing hops through the hash map.
    ///
    /// Look-ups into the hashtable will fail if no appropriate
    /// slot has been found after this amount of hops.
    const MAX_PROBE_HOPS: u32 = 32;

    /// Mutates the value associated with the key if any.
    ///
    /// Returns a reference to the mutated element or
    /// Returns `None` and won't mutate if there is no value
    /// associated to the key.
    pub fn mutate_with<Q, F>(&mut self, key: &Q, f: F) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
        F: FnOnce(&mut V),
    {
        // Use a wrapper closure that wraps the given mutator.
        //
        // This wrapper closure actually works on hashmap entries
        // instead of values directly.
        let wrapped_f = move |entry: &mut Entry<K, V>| {
            match entry {
                Entry::Occupied(occupied) => f(&mut occupied.val),
                Entry::Removed => (),
            }
        };
        self.probe_inspecting(key)
            .and_then(move |probe_index| {
                if let Some(val) = self.entries.get_mut(probe_index) {
                    wrapped_f(val);
                }
                self.entries.get(probe_index)
            })
            .and_then(|entry| {
                match entry {
                    Entry::Occupied(occupied) => Some(&occupied.val),
                    Entry::Removed => None,
                }
            })
    }

    /// Probes for a free or usable slot.
    ///
    /// # Note
    ///
    /// - Uses quadratic probing.
    /// - Returns `(true, _)` if there was a key-match of an already
    ///   occupied slot, returns `(false, _)` if the found slot is empty.
    /// - Returns `(_, n)` if `n` is the found probed index.
    fn probe<Q>(&self, key: &Q, inserting: bool) -> Option<ProbeSlot>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        // Convert the first 4 bytes in the keccak256 hash
        // of the key into a big-endian unsigned integer.
        let probe_start = u32::from_be_bytes(
            slice_as_array4(&(hash::keccak256(key.borrow())[0..4])).expect(
                "[ink_core::HashMap::insert] Error \
                 couldn't convert to probe_start byte array",
            ),
        );
        // We need the hops counter to prevent theroretical endless loop situations.
        let mut probe_hops = 0;
        while probe_hops < Self::MAX_PROBE_HOPS {
            let probe_offset = probe_hops * probe_hops;
            let probe_index = probe_start.wrapping_add(probe_offset);
            match self.entries.get(probe_index) {
                Some(Entry::Occupied(entry)) => {
                    if key == entry.key.borrow() {
                        // Keys match so we can return this probed slot.
                        return Some(ProbeSlot::Occupied(probe_index))
                    }
                }
                None => {
                    if inserting {
                        // We can insert into this slot.
                        return Some(ProbeSlot::Vacant(probe_index))
                    } else {
                        // The searched for element does not exist.
                        return None
                    }
                }
                Some(Entry::Removed) => {
                    // We can insert into this slot.
                    if inserting {
                        return Some(ProbeSlot::Vacant(probe_index))
                    }
                }
            }
            probe_hops += 1;
        }
        None
    }

    /// Probes for a free or usable slot while inserting.
    ///
    /// Returns `None` if there is no mapping for the key.
    ///
    /// # Note
    ///
    /// For more information refer to the `fn probe` documentation.
    fn probe_inserting<Q>(&self, key: &Q) -> Option<ProbeSlot>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.probe(key, true)
    }

    /// Probes an occupied or once occupied slot with the given key.
    ///
    /// # Note
    ///
    /// For more information refer to the `fn probe` documentation.
    fn probe_inspecting<Q>(&self, key: &Q) -> Option<u32>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.probe(key, false).map(|slot| slot.index())
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    ///
    /// # Note
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but Hash and Eq on the borrowed form must match those for the key type.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let probe_index = self.probe_inspecting(key).expect(
            "[ink_core::HashMap::remove] Error: \
             failed at finding a valid entry",
        );
        match self.entries.take(probe_index) {
            Some(Entry::Removed) | None => None,
            Some(Entry::Occupied(OccupiedEntry { val, .. })) => {
                self.len.set(self.len() - 1);
                Some(val)
            }
        }
    }

    /// Returns an immutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but Hash and Eq on the borrowed form must match those for the key type.
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.entry(key) {
            Some(Entry::Removed) | None => None,
            Some(Entry::Occupied(OccupiedEntry { val, .. })) => Some(val),
        }
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but Hash and Eq on the borrowed form must match those for the key type.
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.entry_mut(key) {
            Some(Entry::Removed) | None => None,
            Some(Entry::Occupied(OccupiedEntry { val, .. })) => Some(val),
        }
    }

    /// Returns `true` if there is an entry corresponding to the key in the map.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Returns an immutable reference to the entry corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but Hash and Eq on the borrowed form must match those for the key type.
    fn entry<Q>(&self, key: &Q) -> Option<&Entry<K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(slot) = self.probe_inspecting(key) {
            return self.entries.get(slot)
        }
        None
    }

    /// Returns a mutable reference to the entry corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type,
    /// but Hash and Eq on the borrowed form must match those for the key type.
    fn entry_mut<Q>(&mut self, key: &Q) -> Option<&mut Entry<K, V>>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(slot) = self.probe_inspecting(key) {
            return self.entries.get_mut(slot)
        }
        None
    }
}

impl<'a, K, Q: ?Sized, V> core::ops::Index<&'a Q> for HashMap<K, V>
where
    K: Eq + Hash + Borrow<Q> + scale::Codec,
    V: scale::Codec,
    Q: Eq + Hash,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index).expect(
            "[ink_core::HashMap::index] Error: \
             expected `index` to be within bounds",
        )
    }
}

impl<'a, K, Q: ?Sized, V> core::ops::IndexMut<&'a Q> for HashMap<K, V>
where
    K: Eq + Hash + Borrow<Q> + scale::Codec,
    V: scale::Codec,
    Q: Eq + Hash,
{
    fn index_mut(&mut self, index: &Q) -> &mut Self::Output {
        self.get_mut(index).expect(
            "[ink_core::HashMap::index_mut] Error: \
             expected `index` to be within bounds",
        )
    }
}
