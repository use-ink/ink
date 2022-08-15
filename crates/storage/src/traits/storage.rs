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

use ink_primitives::{
    traits::Storable,
    Key,
};

/// Trait for describing types that can be read and written to storage while all fields occupy
/// only a single storage cell.
///
/// If at least one of the fields in the type occupies its storage cell, this type
/// is considered non-packed.
///
/// # Note
///
/// The trait is automatically implemented for types that implement `scale::Encode`
/// and `scale::Decode` via blank implementation.
///
/// Don't try to implement that trait manually.
pub trait Packed: Storable + scale::Decode + scale::Encode {}

/// Holds storage key for the type.
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`](crate::traits::Packed) types
/// via blank implementation.
pub trait StorageKey {
    /// Storage key of the type.
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
pub trait Item<Key: StorageKey> {
    /// Storable type with storage key inside.
    type Type: Storable;
    /// The storage key that the type prefers. It can be overwritten by an auto-generated storage key.
    type PreferredKey: StorageKey;
}

/// Automatically returns the type that should be used for storing the value.
///
/// The trait is used by codegen to determine which storage key the type should have.
pub trait AutoItem<Key: StorageKey> {
    /// Storable type with storage key inside.
    type Type: Storable;
}

/// A trait to support initialization on the runtime if it cannot pull from the storage.
///
/// It can be in several cases:
/// - The contract doesn't have constructor. That initializer can be alternative for the constructor.
/// - The constructor was not called due to upgrade ability, `Proxy` or `Diamond` pattern.
/// - The storage was moved or corrupted.
///
/// If the trait is not implemented the behavior of the storage is default.
/// It should be first initialized by the constructor.
pub trait OnCallInitializer: Default {
    /// A default instance of the contract is first created. The initialize method
    /// is then called on that instance. There are no restrictions to what a developer
    /// may do during the initialization phase, including doing nothing.
    fn initialize(&mut self);
}
