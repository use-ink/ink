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

use ink_primitives::StorageKey;

/// Returns storage key for the type
pub trait StorageKeyHolder {
    /// Storage key
    const KEY: StorageKey;
}

/// `AtomicGuard<true>` is automatically implemented for all primitive types and atomic structures.
/// It can be used to add requirement for the generic to be atomic.
///
/// `AtomicGuard<false>` is useless bound because every type can implements it without any restriction.
pub trait AtomicGuard<const IS_ATOMIC: bool> {}

/// Describes the type that should be used for storing the value and preferred storage key.
pub trait StorageType<Salt: StorageKeyHolder> {
    /// Type with storage key inside
    type Type: scale::Encode + scale::Decode;
    /// Preferred storage key
    type PreferredKey: StorageKeyHolder;
}

/// Automatically returns the type that should be used for storing the value.
///
/// Trait is used be codegen to use the right storage type.
pub trait AutoStorageType<Salt: StorageKeyHolder> {
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
