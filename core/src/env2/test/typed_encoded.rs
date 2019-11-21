// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use core::{
    any::TypeId,
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
    marker::PhantomData,
};

/// A wrapper around an encoded entity that only allows type safe accesses.
///
/// # Note
///
/// Checks are implemented at runtime.
#[derive(Debug)]
pub struct TypedEncoded<M> {
    /// The bytes of the encoded representation of the type.
    encoded: Vec<u8>,
    /// The unique identifier of the encoded type.
    type_id: TypeId,
    /// Classification marker.
    ///
    /// # Note
    ///
    /// - This shouldn't be the typed that is actually stored as encoded
    ///   representation in `self.encoded` but should primarily be an
    ///   abstract marker type that may be used for classification.
    /// - The idea behind the marker is to say that whenever two instances
    ///   of `TypedEncoded` share a marker they are guaranteed to also have
    ///   a common (but unknown) `type_id` so they can decode to the same
    ///   original type and thus we can allow to interoperate on them.
    ///
    /// # Example
    ///
    /// The `TestEnvInstance` might use one abstract marker for every
    /// of the fundamental SRML types: `Balance`, `AccountId`, `Hash`, etc.
    /// With this and the explicit guarantee that two instances of `TypedEncoded`
    /// with the same abstract marker also share the same (unknown) `type_id`
    /// it is possible to allow them to interoperate.
    marker: PhantomData<fn() -> M>,
}

impl<M> PartialEq<Self> for TypedEncoded<M> {
    fn eq(&self, other: &Self) -> bool {
        if self.type_id == other.type_id
            && self.encoded.as_slice() == other.encoded.as_slice()
        {
            return true
        }
        false
    }
}

impl<M> Eq for TypedEncoded<M> {}

impl<M> PartialOrd<Self> for TypedEncoded<M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.type_id != other.type_id {
            return None
        }
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl<M> Ord for TypedEncoded<M> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other)
            .expect("expect to have same `type_id`")
    }
}

impl<M> Clone for TypedEncoded<M> {
    fn clone(&self) -> Self {
        Self {
            encoded: self.encoded.clone(),
            type_id: self.type_id,
            marker: Default::default(),
        }
    }
}

impl<M> Hash for TypedEncoded<M> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.encoded.hash(state);
        self.type_id.hash(state);
    }
}

/// Marker that indicates untypedness for an instance of `TypedEncoded`.
///
/// # Note
///
/// - We abuse this to initialize instances of `TypedEncoded` where concrete
///   types for them are not yet known.
/// - This allows for a special case where even instances with differing marker
///   types can interoperate as long as at least one of them has `Uninitialized` as marker,
///   although this is type unsafe since we cannot guarantee that the encoding matches
///   the type.
enum Uninitialized {}

impl<M> Default for TypedEncoded<M> {
    /// Creates an uninitialized instance.
    ///
    /// # Note
    ///
    /// This instance can be initialized with a proper value at a later point
    /// using a call to `TypedEncoded::try_initialize`.
    fn default() -> Self {
        Self {
            encoded: Vec::new(),
            type_id: TypeId::of::<Uninitialized>(),
            marker: Default::default(),
        }
    }
}

/// Encountered when trying to initialize an already initialized `TypedEncoded`.
#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyInitialized;

impl<M> TypedEncoded<M> {
    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<V>(&mut self, value: &V) -> Result<(), AlreadyInitialized>
    where
        V: scale::Encode + 'static,
    {
        if self.type_id != TypeId::of::<Uninitialized>() {
            return Err(AlreadyInitialized)
        }
        self.encoded = value.encode();
        self.type_id = TypeId::of::<V>();
        Ok(())
    }
}

impl<M> TypedEncoded<M> {
    /// Returns the encoded bytes.
    fn as_bytes(&self) -> &[u8] {
        &self.encoded
    }
}

/// Encountered when trying to decode a `TypedEncoded` as an invalid type.
#[derive(Debug, PartialEq, Eq)]
pub struct UnmatchingType;

impl<M> TypedEncoded<M> {
    /// Converts back into the original typed origin.
    ///
    /// # Errors
    ///
    /// If the given type doesn't match the origin's real type.
    pub fn try_to_origin<T>(&self) -> Result<T, UnmatchingType>
    where
        T: scale::Decode + 'static,
    {
        if self.type_id != TypeId::of::<T>() {
            return Err(UnmatchingType)
        }
        let decoded = T::decode(&mut self.as_bytes())
            .expect("we should have received an instance of this by encoding; qed");
        Ok(decoded)
    }

    /// Converts back into the original typed origin.
    ///
    /// # Panics
    ///
    /// If the given type doesn't match the origin's real type.
    pub fn to_origin<T>(&self) -> T
    where
        T: scale::Decode + 'static,
    {
        self.try_to_origin()
            .expect("encountered invalid origin type")
    }
}

impl<M> TypedEncoded<M> {
    /// Converts the original typed entity into its encoded representation.
    pub fn from_origin<T>(value: &T) -> Self
    where
        T: scale::Encode + 'static,
    {
        Self {
            encoded: value.encode(),
            type_id: TypeId::of::<T>(),
            marker: Default::default(),
        }
    }

    /// Tries to assign a new value to `self`.
    ///
    /// # Errors
    ///
    /// If the types of the current and new value do not match.
    pub fn try_assign<T>(&mut self, new_value: &T) -> Result<(), UnmatchingType>
    where
        T: scale::Encode + 'static,
    {
        if self.type_id != TypeId::of::<T>() {
            return Err(UnmatchingType)
        }
        self.encoded.clear();
        new_value.encode_to(&mut self.encoded);
        Ok(())
    }

    /// Assigns a new value to `self`.
    ///
    /// # Panics
    ///
    /// If the types of the current and new value do not match.
    pub fn assign<T>(&mut self, new_value: &T)
    where
        T: scale::Encode + 'static,
    {
        self.try_assign(new_value)
            .expect("encountered invalid assignment type")
    }
}
