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

use ink_primitives::Key;

/// Types that implement `scale::Encode` and `scale::Decode` called - **Packed**. Those types
/// support serialization and deserialization into/from storage and occupy only one storage cell.
///
/// All other types - **Non-Packed**.
///
/// # Note
///
/// The trait is automatically implemented for types that implement `scale::Encode`
/// and `scale::Decode` via blank implementation.
///
/// Don't try to implement that trait manually.
pub trait Packed: scale::Decode + scale::Encode {}

/// Every type that wants to be a part of the storage should implement this trait.
/// The trait is used for serialization/deserialization into/from storage.
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`](crate::traits::Packed) types
/// via blank implementation.
pub trait Storable: Sized {
    /// Convert self to a slice and append it to the destination.
    fn encode<T: scale::Output + ?Sized>(&self, dest: &mut T);

    /// Attempt to deserialize the value from input.
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error>;
}

/// Returns storage key for the type
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`](crate::traits::Packed) types
/// via blank implementation.
pub trait KeyHolder {
    /// Storage key of the type
    const KEY: Key;

    /// Returns the storage key.
    fn key(&self) -> Key {
        Self::KEY
    }
}

/// Describes the type that should be used for storing the value and preferred storage key.
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`](crate::traits::Packed) types
/// via blank implementation.
pub trait Item<Salt: KeyHolder> {
    /// Type with storage key inside
    type Type: Storable;
    /// Preferred storage key
    type PreferredKey: KeyHolder;
}

/// Automatically returns the type that should be used for storing the value.
///
/// Trait is used be codegen to use the right storage type.
pub trait AutoItem<Salt: KeyHolder> {
    /// Type with storage key inside
    type Type;
}

/// The contract can implement that trait to support initialization on the runtime
/// if it is unable to pull from the storage.
///
/// It can be in several cases:
/// - The contract doesn't have constructor. That initializer can be alternative for the constructor.
/// - The constructor was not called due to upgrade ability, `Proxy` or `Diamond` pattern.
/// - The storage was moved or corrupted.
///
/// If the trait is not implemented the behavior of the storage is default.
/// It should be first initialized by the constructor.
pub trait OnCallInitializer: Default {
    /// `Default::default` creates the instance of the contract.
    /// After the `initialize` method is called on that instance.
    /// The developer can do everything that he wants during initialization or do nothing.
    fn initialize(&mut self);
}
