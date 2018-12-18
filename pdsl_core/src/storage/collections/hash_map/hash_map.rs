use crate::storage::{
	Key,
	cell::SyncCell,
	chunk::SyncChunk
};
use crate::hash;
use crate::Setup;

use std::borrow::Borrow;
use std::hash::Hash;

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
pub struct HashMap<K, V> {
	/// The storage key to the length of this storage map.
	len: SyncCell<u32>,
	/// The first half of the entry buffer is equal to the key,
	/// the second half will be replaced with the respective
	/// hash of any given key upon usage.
	///
	/// Afterwards this value is hashed again and used as key
	/// into the contract storage.
	entries: SyncChunk<Entry<K, V>>,
}

/// An entry of a storage map.
///
/// This can either store the entries key and value
/// or represent an entry that was removed after it
/// has been occupied with key and value.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(parity_codec_derive::Encode, parity_codec_derive::Decode)]
pub enum Entry<K, V> {
	/// An occupied slot with a key and a value.
	Occupied(OccupiedEntry<K, V>),
	/// A removed slot that was occupied before.
	Removed,
}

/// An occupied entry of a storage map.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(parity_codec_derive::Encode, parity_codec_derive::Decode)]
pub struct OccupiedEntry<K, V> {
	/// The entry's key.
	key: K,
	/// The entry's value.
	val: V,
}

impl<K, V> Setup for HashMap<K, V> {
	fn setup(&mut self) {
		self.len.set(0);
	}
}

impl<K, V> HashMap<K, V> {
	/// Creates a new storage hash map for the given key.
	///
	/// # Safety
	///
	/// This is an inherently unsafe operation since it does not check
	/// for the storage hash map's invariances, such as
	///
	/// - Is the storage region determined by the given key aliasing?
	/// - Is the storage region correctly formatted to be used as storage hash map?
	///
	/// Users should not use this routine directly if possible.
	pub unsafe fn new_unchecked(key: Key) -> Self {
		Self{
			len: SyncCell::new_unchecked(key),
			entries: {
				SyncChunk::new_unchecked(
					Key::with_offset(key, 1),
					u32::max_value()
				)
			},
		}
	}

	/// Returns the number of key-value pairs in the map.
	pub fn len(&self) -> u32 {
		*self
			.len
			.get()
			.unwrap_or(&0)
	}

	/// Returns `true` if the map contains no elements.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// Converts the given bytes into a `u32` value.
///
/// The first byte in the array will be the most significant byte.
fn bytes4_to_u32(bytes: [u8; 4]) -> u32 {
	let mut res = 0;
	res |= (bytes[0] as u32) << 24;
	res |= (bytes[1] as u32) << 16;
	res |= (bytes[2] as u32) <<  8;
	res |= (bytes[3] as u32) <<  0;
	res
}

/// Converts the given slice into an array with fixed size of 4.
///
/// Returns `None` if the slice's length is not 4.
fn slice_as_array4<T>(bytes: &[T]) -> Option<[T; 4]>
where
	T: Default + Copy
{
	if bytes.len() != 4 {
		return None
	}
	let mut array = [T::default(); 4];
	for i in 0..4 {
		array[i] = bytes[i];
	}
	Some(array)
}

impl<K, V> HashMap<K, V>
where
	K: parity_codec::Codec + Hash + Eq,
	V: parity_codec::Codec,
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
				self
					.entries
					.replace(
						probe_index,
						Entry::Occupied(OccupiedEntry{key, val}),
					)
					.unwrap()
					.and_then(|old| {
						match old {
							Entry::Occupied(OccupiedEntry{val, ..}) => Some(val),
							Entry::Removed => None,
						}
					})
			}
			Some(ProbeSlot::Vacant(probe_index)) => {
				// We can insert into this slot.
				self.len.set(self.len() + 1);
				self
					.entries
					.set(
						probe_index,
						Entry::Occupied(OccupiedEntry{key, val})
					)
					.unwrap();
				None
			}
			None => {
				panic!(
					"[pdsl_core::HashMap::insert] Error: failed finding a valid entry"
				)
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
			| ProbeSlot::Vacant(index)
			| ProbeSlot::Occupied(index) => *index
		}
	}
}

impl<K, V> HashMap<K, V>
where
	K: parity_codec::Codec,
	V: parity_codec::Codec,
{
	/// The maximum amount of probing hops through the hash map.
	///
	/// Look-ups into the hashtable will fail if no appropriate
	/// slot has been found after this amount of hops.
	const MAX_PROBE_HOPS: u32 = 32;

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
		let probe_start = bytes4_to_u32(
			slice_as_array4(
				&(hash::keccak256(key.borrow())[0..4])
			).expect(
				"[pdsl_core::HashMap::insert] Error \
				 couldn't convert to probe_start byte array"
			)
		);
		// We need the hops counter to prevent theroretical endless loop situations.
		let mut probe_hops = 0;
		while probe_hops < Self::MAX_PROBE_HOPS {
			let probe_offset = probe_hops * probe_hops;
			let probe_index = probe_start.wrapping_add(probe_offset);
			match self.entries.get(probe_index).unwrap() {
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
		Q: Hash + Eq
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
		self
			.probe(key, false)
			.map(|slot| slot.index())
	}

	/// Removes a key from the map,
	/// returning the value at the key if the key was previously in the map.
	///
	/// # Note
	///
	/// The key may be any borrowed form of the map's key type,
	/// but Hash and Eq on the borrowed form must match those for the key type.
	pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
	where
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized
	{
		let probe_index = self
			.probe_inspecting(key)
			.expect(
				"[pdsl_core::HashMap::remove] Error: \
				 failed at finding a valid entry"
			);
		match self.entries.remove(probe_index).unwrap() {
			Some(Entry::Removed) | None => None,
			Some(Entry::Occupied(OccupiedEntry{val, ..})) => {
				self.len.set(self.len() - 1);
				Some(val)
			},
		}
	}

	/// Returns the value corresponding to the key.
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
			Some(Entry::Occupied(OccupiedEntry{val, ..})) => Some(val),
		}
	}

	/// Returns `true` if there is an entry corresponding to the key in the map.
	pub fn contains_key<Q>(&self, key: &Q) -> bool
	where
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		match self.get(key) {
			Some(_) => true,
			None => false,
		}
	}

	/// Returns the entry corresponding to the key.
	///
	/// The key may be any borrowed form of the map's key type,
	/// but Hash and Eq on the borrowed form must match those for the key type.
	fn entry<Q>(&self, key: &Q) -> Option<&Entry<K, V>>
	where
		K: Borrow<Q>,
		Q: Hash + Eq + ?Sized,
	{
		if let Some(slot) = self.probe_inspecting(key) {
			return self.entries.get(slot).unwrap()
		}
		None
	}
}
