use crate::traits::{
    AtomicStatus,
    AutoKey,
    ManualKey,
    StorageKeyHolder,
    StorageType,
};
use core::marker::PhantomData;
use ink_primitives::StorageKey;
use scale::{
    Decode,
    Encode,
};

/// TODO: Add comment
pub struct StorageValue<V, KeyType: StorageKeyHolder = AutoKey> {
    _marker: PhantomData<fn() -> (V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<V, KeyType: StorageKeyHolder> Default for StorageValue<V, KeyType> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType: StorageKeyHolder> StorageValue<V, KeyType> {
    /// TODO: Add comment
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType: StorageKeyHolder> core::fmt::Debug for StorageValue<V, KeyType> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("StorageValue")
            .field("storage_key", &KeyType::KEY)
            .finish()
    }
}

impl<V: Decode, KeyType: StorageKeyHolder> StorageValue<V, KeyType> {
    /// TODO: Add comment
    pub fn get() -> V {
        ink_env::get_storage_value::<V>(&KeyType::KEY)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to get storage value from key {}: {:?}",
                    KeyType::KEY,
                    error
                )
            })
            .unwrap()
    }
}

impl<V: Decode + Default, KeyType: StorageKeyHolder> StorageValue<V, KeyType> {
    /// TODO: Add comment
    pub fn get_or_default() -> V {
        ink_env::get_storage_value::<V>(&KeyType::KEY)
            .unwrap_or_default()
            .unwrap()
    }
}

impl<V: Encode, KeyType: StorageKeyHolder> StorageValue<V, KeyType> {
    /// TODO: Add comment
    pub fn set(value: &V) {
        ink_env::set_storage_value::<V>(&KeyType::KEY, value)
    }
}

impl<
        V: StorageType<ManualKey<MANUAL_KEY, ManualSalt>>,
        Salt: StorageKeyHolder,
        const MANUAL_KEY: StorageKey,
        ManualSalt: StorageKeyHolder,
    > StorageType<Salt> for StorageValue<V, ManualKey<MANUAL_KEY, ManualSalt>>
{
    type Type = StorageValue<
        <V as StorageType<ManualKey<MANUAL_KEY, ManualSalt>>>::Type,
        ManualKey<MANUAL_KEY, ManualSalt>,
    >;
}

impl<V: StorageType<ManualKey<0, Salt>>, Salt: StorageKeyHolder> StorageType<Salt>
    for StorageValue<V, AutoKey>
{
    type Type =
        StorageValue<<V as StorageType<ManualKey<0, Salt>>>::Type, ManualKey<0, Salt>>;
}

impl<V, KeyType: StorageKeyHolder> AtomicStatus for StorageValue<V, KeyType> {
    const IS_ATOMIC: bool = false;
}
