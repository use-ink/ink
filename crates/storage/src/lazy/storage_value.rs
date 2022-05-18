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

//! A simple wrapper around type to store it in a separate storage cell.
//! This wrapper doesn't actually "own" any data.

use crate::traits::{
    pull_storage,
    push_storage,
    AutoKey,
    StorageKeyHolder,
    StorageType,
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

/// A simple wrapper around type to store it in a separate storage cell under own storage key.
/// If you want to update the value, first you need to `get` it, update, and after `set`.
///
/// # Important
///
/// The wrapper requires its own pre-defined storage key where to store value. By default,
/// it is [`AutoKey`](crate::traits::AutoKey) and during compilation is calculated based on
/// the name of the structure and the field. But anyone can specify its storage key
/// via [`ManualKey`](crate::traits::ManualKey).
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
/// use ink_storage::{traits::ManualKey, StorageValue};
///
/// #[ink(storage)]
/// #[derive(Default)]
/// pub struct MyContract {
///     owner: StorageValue<AccountId>,
///     balance: StorageValue<Balance, ManualKey<123>>,
/// }
///
/// impl MyContract {
///     #[ink(constructor)]
///     pub fn new() -> Self {
///         let mut instance = Self::default();
///         instance.new_init();
///         instance
///     }
///
///     /// Default initializes the contract.
///     fn new_init(&mut self) {
///         let caller = Self::env().caller();
///         self.owner.set(&caller);
///         self.balance.set(&123456);
///     }
///
/// #   #[ink(message)]
/// #   pub fn my_message(&self) { }
/// }
/// # }
/// ```
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct StorageValue<V, KeyType: StorageKeyHolder = AutoKey> {
    _marker: PhantomData<fn() -> (V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<V, KeyType> Default for StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType> StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    /// Creates a new empty `StorageValue`.
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType> core::fmt::Debug for StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("StorageValue")
            .field("storage_key", &KeyType::KEY)
            .finish()
    }
}

impl<V, KeyType> StorageValue<V, KeyType>
where
    V: Decode,
    KeyType: StorageKeyHolder,
{
    /// Get the `value` from the contract storage.
    ///
    /// Panics if no `value` exists.
    pub fn get(&self) -> V {
        pull_storage(&KeyType::KEY)
    }
}

impl<V, KeyType> StorageValue<V, KeyType>
where
    V: Decode + Default,
    KeyType: StorageKeyHolder,
{
    /// Get the `value` from the contract storage.
    ///
    /// Returns `Default::default()` if no `value` exists.
    pub fn get_or_default(&self) -> V {
        ink_env::get_storage_value::<V>(&KeyType::KEY)
            .unwrap_or_default()
            .unwrap_or_default()
    }
}

impl<V, KeyType> StorageValue<V, KeyType>
where
    V: Encode,
    KeyType: StorageKeyHolder,
{
    /// Sets the given `value` to the contract storage.
    pub fn set(&mut self, value: &V) {
        push_storage(value, &KeyType::KEY);
    }
}

impl<V, Salt, InnerSalt> StorageType<Salt> for StorageValue<V, InnerSalt>
where
    Salt: StorageKeyHolder,
    InnerSalt: StorageKeyHolder,
    V: StorageType<Salt>,
{
    type Type = StorageValue<V::Type, Salt>;
    type PreferredKey = InnerSalt;
}

impl<V, KeyType> Encode for StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    fn encode_to<T: Output + ?Sized>(&self, _dest: &mut T) {}
}

impl<V, KeyType> Decode for StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
    }
}

impl<V, KeyType> StorageKeyHolder for StorageValue<V, KeyType>
where
    KeyType: StorageKeyHolder,
{
    const KEY: StorageKey = KeyType::KEY;
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::StorageLayout;
    use ink_metadata::layout::{
        Layout,
        LayoutKey,
        RootLayout,
    };

    impl<V, KeyType> StorageLayout for StorageValue<V, KeyType>
    where
        V: StorageLayout + scale_info::TypeInfo + 'static,
        KeyType: StorageKeyHolder + scale_info::TypeInfo + 'static,
    {
        fn layout(_: &StorageKey) -> Layout {
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
            let mut storage: StorageValue<u8, ManualKey<123>> = StorageValue::new();
            storage.set(&2);
            assert_eq!(storage.get(), 2);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_or_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: StorageValue<u8, ManualKey<123>> = StorageValue::new();
            assert_eq!(storage.get_or_default(), 0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "storage entry was empty")]
    fn gets_failes_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: StorageValue<u8, ManualKey<123>> = StorageValue::new();
            storage.get();

            Ok(())
        })
        .unwrap()
    }
}
