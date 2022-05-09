use crate::traits::{
    pull_storage,
    push_storage,
    AutoKey,
    ManualKey,
    StorageKeyHolder,
    StorageType,
    StorageType2,
};
use core::marker::PhantomData;
use ink_primitives::StorageKey;
use scale::{
    Decode,
    Encode,
    Error,
    Input,
    Output,
};

/// TODO: Add comment
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
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
        pull_storage(&KeyType::KEY)
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
        push_storage(value, &KeyType::KEY)
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

impl<V: StorageType2, Salt: StorageKeyHolder> StorageType2 for StorageValue<V, Salt> {
    type Type<SaltInner: StorageKeyHolder> = StorageValue<V::Type<SaltInner>, SaltInner>;
    type PreferredKey = Salt;
}

impl<V: StorageType<ManualKey<0, Salt>>, Salt: StorageKeyHolder> StorageType<Salt>
    for StorageValue<V, AutoKey>
{
    type Type =
        StorageValue<<V as StorageType<ManualKey<0, Salt>>>::Type, ManualKey<0, Salt>>;
}

impl<V, KeyType: StorageKeyHolder> Encode for StorageValue<V, KeyType> {
    fn encode_to<T: Output + ?Sized>(&self, _dest: &mut T) {}
}

impl<V, KeyType: StorageKeyHolder> Decode for StorageValue<V, KeyType> {
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
    }
}
