use crate::traits::{
    AtomicGuard,
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
    Error,
    Input,
    Output,
};

/// TODO: Add comment
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StorageMapping<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder = AutoKey> {
    _marker: PhantomData<fn() -> (K, V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> Default
    for StorageMapping<K, V, KeyType>
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> StorageMapping<K, V, KeyType> {
    /// TODO: Add comment
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> core::fmt::Debug
    for StorageMapping<K, V, KeyType>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("StorageMapping")
            .field("storage_key", &KeyType::KEY)
            .finish()
    }
}

impl<K, V, KeyType> StorageMapping<K, V, KeyType>
where
    K: Encode,
    V: AtomicGuard<true> + Encode + Decode,
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
        ink_env::get_contract_storage::<V>(&root_key).unwrap_or_else(|error| {
            panic!(
                "failed to get packed from root key {}: {:?}",
                root_key, error
            )
        })
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
        V: AtomicGuard<true>,
        Salt: StorageKeyHolder,
        const MANUAL_KEY: StorageKey,
        ManualSalt: StorageKeyHolder,
    > StorageType<Salt> for StorageMapping<K, V, ManualKey<MANUAL_KEY, ManualSalt>>
{
    type Type = StorageMapping<K, V, ManualKey<MANUAL_KEY, ManualSalt>>;
}

impl<K, V: AtomicGuard<true>, Salt: StorageKeyHolder> StorageType<Salt>
    for StorageMapping<K, V, AutoKey>
{
    type Type = StorageMapping<K, V, ManualKey<0, Salt>>;
}

impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> Encode
    for StorageMapping<K, V, KeyType>
{
    fn encode_to<T: Output + ?Sized>(&self, _dest: &mut T) {}
}

impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> Decode
    for StorageMapping<K, V, KeyType>
{
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
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

    impl<K, V: AtomicGuard<true>, KeyType: StorageKeyHolder> StorageLayout
        for StorageMapping<K, V, KeyType>
    where
        K: scale_info::TypeInfo + 'static,
        V: scale_info::TypeInfo + 'static,
        KeyType: scale_info::TypeInfo + 'static,
    {
        fn layout(_key: &StorageKey) -> Layout {
            Layout::Cell(CellLayout::new::<Self>(LayoutKey::from(&KeyType::KEY)))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: StorageMapping<u8, _> = StorageMapping::new();
            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: StorageMapping<u8, u8> = StorageMapping::new();
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mut mapping: StorageMapping<u8, u8> = StorageMapping::new();

            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            // When
            mapping.remove(&1);

            // Then
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn can_clear_unexistent_entries() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            // Given
            let mapping: StorageMapping<u8, u8> = StorageMapping::new();

            // When
            mapping.remove(&1);

            // Then
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }
}
