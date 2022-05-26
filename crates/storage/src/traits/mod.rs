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

//! Traits and interfaces to operate with storage entities.
//!
//! Generally a type is said to be a storage entity if it implements the
//! `Item` trait. This defines certain constants and routines in order
//! to tell a smart contract how to load and store instances of this type
//! from and to the contract's storage.
//!
//! The `Packed` shows that the type is packed and can be stored
//! into single storage cell. Some collections works only with packed structures.

mod impls;
mod storage;

#[cfg(feature = "std")]
mod layout;

#[macro_use]
#[doc(hidden)]
pub mod pull_or_init;

#[cfg(feature = "std")]
pub use self::layout::{
    LayoutCryptoHasher,
    StorageLayout,
};
pub use self::{
    impls::storage::{
        AutoKey,
        ManualKey,
        ResolverKey,
    },
    storage::{
        AutoItem,
        Item,
        KeyHolder,
        OnCallInitializer,
        Packed,
        Storable,
    },
};
use ink_primitives::Key;
pub use ink_storage_derive::{
    Item,
    KeyHolder,
    Storable,
    StorageLayout,
};
use scale::{
    Decode,
    Encode,
};

#[repr(transparent)]
pub(crate) struct DecodeWrapper<S: Storable>(pub S);

impl<S: Storable> Decode for DecodeWrapper<S> {
    #[inline(always)]
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        Ok(Self(S::decode(input)?))
    }
}

/// Pulls an instance of type `T` from the contract storage using decode and its storage key.
pub fn pull_storage<T>(key: &Key) -> T
where
    T: Storable,
{
    match ink_env::get_contract_storage::<(), DecodeWrapper<T>>(key, None) {
        Ok(Some(wrapper)) => wrapper.0,
        Ok(None) => panic!("storage entry was empty"),
        Err(_) => panic!("could not properly decode storage entry"),
    }
}

#[repr(transparent)]
pub(crate) struct EncodeWrapper<'a, S: Storable>(pub &'a S);

impl<'a, S: Storable> Encode for EncodeWrapper<'a, S> {
    #[inline(always)]
    fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
        self.0.encode(dest)
    }
}

/// Pushes the entity to the contract storage using encode and storage key.
pub fn push_storage<T>(entity: &T, key: &Key) -> Option<u32>
where
    T: Storable,
{
    ink_env::set_contract_storage::<(), EncodeWrapper<T>>(
        key,
        None,
        &EncodeWrapper(entity),
    )
}
