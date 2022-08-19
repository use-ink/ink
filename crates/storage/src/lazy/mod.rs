// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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

//! Low-level collections and data structures to manage storage entities in the
//! persisted contract storage.
//!
//! These low-level collections are not aware of the elements they manage thus
//! extra care has to be taken when operating directly on them.

mod mapping;

#[doc(inline)]
pub use self::mapping::Mapping;

use crate::traits::{
    push_storage,
    AutoKey,
    StorableHint,
    StorageKey,
};
use core::marker::PhantomData;
use ink_primitives::{
    traits::Storable,
    Key,
};
use scale::{
    Error,
    Input,
    Output,
};

/// A simple wrapper around a type to store it in a separate storage cell under its own storage key.
/// If you want to update the value, first you need to [`get`](crate::Lazy::get) it, update the
/// value, and then call [`set`](crate::Lazy::set) with the new value.
///
/// # Important
///
/// The wrapper requires its own pre-defined storage key in order to determine where it stores
/// value. By default, the is automatically calculated using [`AutoKey`](crate::traits::AutoKey)
/// during compilation. However, anyone can specify a storage key using
/// [`ManualKey`](crate::traits::ManualKey). Specifying the storage key can be helpful for
/// upgradeable contracts or you want to be resistant to future changes of storage
/// key calculation strategy.
///
/// # Note
///
/// If the contract has two or more `Lazy` with the same storage key, modifying the value of one
/// of them will modify others.
///
/// This is an example of how you can do this:
/// ```rust
/// # use ink_lang as ink;
/// # use ink_env::{
/// #     Environment,
/// #     DefaultEnvironment,
/// # };
/// # type AccountId = <DefaultEnvironment as Environment>::AccountId;
///
/// # #[ink::contract]
/// # mod my_module {
/// use ink_storage::{traits::ManualKey, Lazy};
///
/// #[ink(storage)]
/// #[derive(Default)]
/// pub struct MyContract {
///     owner: Lazy<AccountId>,
///     balance: Lazy<Balance, ManualKey<123>>,
/// }
///
/// impl MyContract {
///     #[ink(constructor)]
///     pub fn new() -> Self {
///         let mut instance = Self::default();
///         let caller = Self::env().caller();
///         instance.owner.set(&caller);
///         instance.balance.set(&123456);
///         instance
///     }
///
/// #   #[ink(message)]
/// #   pub fn my_message(&self) { }
/// }
/// # }
/// ```
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Lazy<V, KeyType: StorageKey = AutoKey> {
    _marker: PhantomData<fn() -> (V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<V, KeyType> Default for Lazy<V, KeyType>
where
    KeyType: StorageKey,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    KeyType: StorageKey,
{
    /// Creates a new empty `Lazy`.
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType> core::fmt::Debug for Lazy<V, KeyType>
where
    KeyType: StorageKey,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Lazy").field("key", &KeyType::KEY).finish()
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    V: Storable,
    KeyType: StorageKey,
{
    /// Reads the `value` from the contract storage, if it exists.
    pub fn get(&self) -> Option<V> {
        match ink_env::get_contract_storage::<Key, V>(&KeyType::KEY) {
            Ok(Some(value)) => Some(value),
            _ => None,
        }
    }

    /// Writes the given `value` to the contract storage.
    pub fn set(&mut self, value: &V) {
        push_storage(&KeyType::KEY, value);
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    V: Storable + Default,
    KeyType: StorageKey,
{
    /// Reads the `value` from the contract storage.
    ///
    /// Returns the default value for the storage type if no `value` exists.
    pub fn get_or_default(&self) -> V {
        match ink_env::get_contract_storage::<Key, V>(&KeyType::KEY) {
            Ok(Some(value)) => value,
            _ => Default::default(),
        }
    }
}

impl<V, KeyType> Storable for Lazy<V, KeyType>
where
    KeyType: StorageKey,
{
    #[inline(always)]
    fn encode<T: Output + ?Sized>(&self, _dest: &mut T) {}

    #[inline(always)]
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
    }
}

impl<V, Key, InnerKey> StorableHint<Key> for Lazy<V, InnerKey>
where
    Key: StorageKey,
    InnerKey: StorageKey,
    V: StorableHint<Key>,
{
    type Type = Lazy<V::Type, Key>;
    type PreferredKey = InnerKey;
}

impl<V, KeyType> StorageKey for Lazy<V, KeyType>
where
    KeyType: StorageKey,
{
    const KEY: Key = KeyType::KEY;
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        Layout,
        LayoutKey,
        RootLayout,
    };

    impl<V, KeyType> StorageLayout for Lazy<V, KeyType>
    where
        V: StorageLayout + scale_info::TypeInfo + 'static,
        KeyType: StorageKey + scale_info::TypeInfo + 'static,
    {
        fn layout(_: &Key) -> Layout {
            Layout::Root(RootLayout::new(
                LayoutKey::from(&KeyType::KEY),
                <V as StorageLayout>::layout(&KeyType::KEY),
            ))
        }
    }
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ManualKey;

    #[test]
    fn set_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut storage: Lazy<u8> = Lazy::new();
            storage.set(&2);
            assert_eq!(storage.get(), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn set_and_get_work_for_two_lazy_with_same_manual_key() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut storage: Lazy<u8, ManualKey<123>> = Lazy::new();
            storage.set(&2);

            let storage2: Lazy<u8, ManualKey<123>> = Lazy::new();
            assert_eq!(storage2.get(), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_or_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: Lazy<u8> = Lazy::new();
            assert_eq!(storage.get_or_default(), 0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_returns_none_if_no_value_was_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: Lazy<u8> = Lazy::new();
            assert_eq!(storage.get(), None);

            Ok(())
        })
        .unwrap()
    }
}
