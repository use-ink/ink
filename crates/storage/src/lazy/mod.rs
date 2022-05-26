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
//! The low-level collections are mainly used as building blocks for internals
//! of other higher-level storage collections.
//!
//! These low-level collections are not aware of the elements they manage thus
//! extra care has to be taken when operating directly on them.

mod mapping;

#[doc(inline)]
pub use self::mapping::Mapping;

use crate::traits::{
    pull_storage,
    push_storage,
    AutoKey,
    DecodeWrapper,
    Item,
    KeyHolder,
    Storable,
};
use core::marker::PhantomData;
use ink_primitives::Key;
use scale::{
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
pub struct Lazy<V, KeyType: KeyHolder = AutoKey> {
    _marker: PhantomData<fn() -> (V, KeyType)>,
}

/// We implement this manually because the derived implementation adds trait bounds.
impl<V, KeyType> Default for Lazy<V, KeyType>
where
    KeyType: KeyHolder,
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    KeyType: KeyHolder,
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
    KeyType: KeyHolder,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Lazy").field("key", &KeyType::KEY).finish()
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    V: Storable,
    KeyType: KeyHolder,
{
    /// Get the `value` from the contract storage.
    ///
    /// Panics if no `value` exists.
    pub fn get(&self) -> V {
        pull_storage(&KeyType::KEY)
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    V: Storable + Default,
    KeyType: KeyHolder,
{
    /// Get the `value` from the contract storage.
    ///
    /// Returns `Default::default()` if no `value` exists.
    pub fn get_or_default(&self) -> V {
        match ink_env::get_contract_storage::<(), DecodeWrapper<V>>(&KeyType::KEY, None) {
            Ok(Some(wrapper)) => wrapper.0,
            _ => Default::default(),
        }
    }
}

impl<V, KeyType> Lazy<V, KeyType>
where
    V: Storable,
    KeyType: KeyHolder,
{
    /// Sets the given `value` to the contract storage.
    pub fn set(&mut self, value: &V) {
        push_storage(value, &KeyType::KEY);
    }
}

impl<V, KeyType> Storable for Lazy<V, KeyType>
where
    KeyType: KeyHolder,
{
    #[inline(always)]
    fn encode<T: Output + ?Sized>(&self, _dest: &mut T) {}

    #[inline(always)]
    fn decode<I: Input>(_input: &mut I) -> Result<Self, Error> {
        Ok(Default::default())
    }
}

impl<V, Salt, InnerSalt> Item<Salt> for Lazy<V, InnerSalt>
where
    Salt: KeyHolder,
    InnerSalt: KeyHolder,
    V: Item<Salt>,
{
    type Type = Lazy<V::Type, Salt>;
    type PreferredKey = InnerSalt;
}

impl<V, KeyType> KeyHolder for Lazy<V, KeyType>
where
    KeyType: KeyHolder,
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
        KeyType: KeyHolder + scale_info::TypeInfo + 'static,
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
            let mut storage: Lazy<u8, ManualKey<123>> = Lazy::new();
            storage.set(&2);
            assert_eq!(storage.get(), 2);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_or_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: Lazy<u8, ManualKey<123>> = Lazy::new();
            assert_eq!(storage.get_or_default(), 0);

            Ok(())
        })
        .unwrap()
    }

    #[test]
    #[should_panic(expected = "storage entry was empty")]
    fn gets_failes_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let storage: Lazy<u8, ManualKey<123>> = Lazy::new();
            storage.get();

            Ok(())
        })
        .unwrap()
    }
}
