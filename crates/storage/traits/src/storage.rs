// Copyright (C) Use Ink (UK) Ltd.
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

/// Trait for representing types which can be read and written to storage.
///
/// This trait is not the same as the [`scale::Codec`]. Each type that implements
/// [`scale::Codec`] are storable by default and transferable between contracts.
/// But not each storable type is transferable.
pub trait Storable: Sized {
    /// Convert self to a slice and append it to the destination.
    fn encode<T: scale::Output + ?Sized>(&self, dest: &mut T);

    /// Attempt to deserialize the value from input.
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error>;

    /// The exact number of bytes this type consumes in the encoded form.
    fn encoded_size(&self) -> usize;
}

/// Types which implement `scale::Encode` and `scale::Decode` are `Storable` by default
/// because they can be written directly into the storage cell.
impl<P> Storable for P
where
    P: scale::Codec,
{
    #[inline]
    fn encode<T: scale::Output + ?Sized>(&self, dest: &mut T) {
        scale::Encode::encode_to(self, dest)
    }

    #[inline]
    fn decode<I: scale::Input>(input: &mut I) -> Result<Self, scale::Error> {
        scale::Decode::decode(input)
    }

    #[inline]
    fn encoded_size(&self) -> usize {
        <P as scale::Encode>::encoded_size(self)
    }
}

/// Decode and consume all the given input data.
///
/// If not all data is consumed, an error is returned.
pub fn decode_all<T: Storable>(input: &mut &[u8]) -> Result<T, scale::Error> {
    let res = <T as Storable>::decode(input)?;

    if input.is_empty() {
        Ok(res)
    } else {
        Err("Input buffer has still data left after decoding!".into())
    }
}

pub(crate) mod private {
    /// Seals the implementation of `Packed`.
    pub trait Sealed {}
}

/// Trait for describing types that can be read and written to storage while all fields
/// occupy only a single storage cell.
///
/// If at least one of the fields in the type occupies its own storage cell, this type
/// is considered non-packed.
///
/// # Note
///
/// The trait is automatically implemented for types that implement [`scale::Codec`]
/// via blanket implementation.
pub trait Packed: Storable + scale::Codec + private::Sealed {}

/// Holds storage key for the type.
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`] types
/// via blanket implementation.
pub trait StorageKey {
    /// Storage key of the type.
    const KEY: Key;

    /// Returns the storage key.
    fn key(&self) -> Key {
        Self::KEY
    }
}

/// Describes the type that should be used for storing the value and preferred storage
/// key.
///
/// # Note
///
/// The trait is automatically implemented for [`Packed`] types
/// via blanket implementation.
pub trait StorableHint<Key: StorageKey> {
    /// Storable type with storage key inside.
    type Type: Storable;
    /// The storage key that the type prefers. It can be overwritten by an auto-generated
    /// storage key.
    type PreferredKey: StorageKey;
}

/// Automatically returns the type that should be used for storing the value.
///
/// The trait is used by codegen to determine which storage key the type should have.
pub trait AutoStorableHint<Key: StorageKey> {
    /// Storable type with storage key inside.
    type Type: Storable;
}
