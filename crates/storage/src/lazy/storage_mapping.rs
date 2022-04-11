use crate::traits::{
    AtomicStatus,
    AutoKey,
    ManualKey,
    StorageKeyHolder,
    StorageType,
};
use core::marker::PhantomData;
use ink_env::hash::{
    Blake2x256,
    HashOutput,
};
use ink_primitives::{
    Key,
    StorageKey,
};
use scale::{
    Decode,
    Encode,
};

/// TODO: Add comment
pub struct StorageMapping<K, V, KeyType: StorageKeyHolder = AutoKey> {
    _marker: PhantomData<fn() -> (K, V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<K, V, KeyType: StorageKeyHolder> Default for StorageMapping<K, V, KeyType> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V, KeyType: StorageKeyHolder> StorageMapping<K, V, KeyType> {
    /// TODO: Add comment
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V, KeyType: StorageKeyHolder> core::fmt::Debug for StorageMapping<K, V, KeyType> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("StorageMapping")
            .field("storage_key", &KeyType::KEY)
            .finish()
    }
}

impl<K, V, KeyType> StorageMapping<K, V, KeyType>
where
    K: Encode,
    V: Encode + Decode,
    KeyType: StorageKeyHolder,
{
    /// Insert the given `value` to the contract storage.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: Q, value: &R)
    where
        Q: scale::EncodeLike<K>,
        R: scale::EncodeLike<V>,
    {
        ink_env::set_contract_storage(&self.storage_key(&key), value);
    }

    /// Get the `value` at `key` from the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        let root_key = self.storage_key(&key);
        ink_env::get_contract_storage(&root_key)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to get packed from root key {}: {:?}",
                    root_key, error
                )
            })
            .unwrap()
    }

    /// Clears the value at `key` from storage.
    pub fn remove<Q>(&self, key: Q)
    where
        Q: scale::EncodeLike<K>,
    {
        ink_env::clear_contract_storage(&self.storage_key(&key));
    }

    /// Returns a `Key` pointer used internally by the storage API.
    ///
    /// This key is a combination of the `Mapping`'s internal `offset_key`
    /// and the user provided `key`.
    fn storage_key<Q>(&self, key: &Q) -> Key
    where
        Q: scale::EncodeLike<K>,
    {
        let encodedable_key = (key, &KeyType::KEY);
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        ink_env::hash_encoded::<Blake2x256, _>(&encodedable_key, &mut output);
        output.into()
    }
}

impl<
        K,
        V: AtomicStatus,
        const KEY: StorageKey,
        Salt: StorageKeyHolder,
        const MANUAL_KEY: StorageKey,
        ManualSalt: StorageKeyHolder,
    > StorageType<KEY, Salt, true>
    for StorageMapping<K, V, ManualKey<MANUAL_KEY, ManualSalt>>
{
    type Type = StorageMapping<K, V, ManualKey<MANUAL_KEY, ManualSalt>>;
}

impl<K, V: AtomicStatus, const KEY: StorageKey, Salt: StorageKeyHolder>
    StorageType<KEY, Salt, true> for StorageMapping<K, V, AutoKey>
{
    type Type = StorageMapping<K, V, ManualKey<KEY, Salt>>;
}

impl<K, V: AtomicStatus, KeyType: StorageKeyHolder> AtomicStatus
    for StorageMapping<K, V, KeyType>
{
    const IS_ATOMIC: bool = false;
    const INNER_IS_ATOMIC: bool = V::IS_ATOMIC;
}
