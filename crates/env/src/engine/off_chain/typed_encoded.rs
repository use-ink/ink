// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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

use super::OffChainError;
use crate::Error;
use core::{
    any::TypeId,
    cmp::Ordering,
    hash::{
        Hash,
        Hasher,
    },
    marker::PhantomData,
};
use derive_more::From;

/// A wrapper around an encoded entity that only allows type safe accesses.
///
/// # Note
///
/// Checks are implemented at runtime.
#[derive(Debug, Clone)]
pub struct TypedEncoded<T> {
    /// The bytes of the encoded representation of the type.
    encoded: Vec<u8>,
    /// The unique identifier of the encoded type.
    ///
    /// # Note
    ///
    /// - If this is `None` it means that the instance is currently untyped
    /// and will take over any given type upon the first typed interaction.
    /// - This is needed since instances of `TypedEncoded` are going to be used
    /// in static memory where it is not possible to decide about the used types
    /// given by `Environment` at initialization.
    type_id: Option<TypeId>,
    /// Classification marker.
    ///
    /// # Note
    ///
    /// - This should not be the typed that is actually stored as encoded
    ///   representation in `self.encoded` but should primarily be an
    ///   abstract marker type that may be used for classification.
    /// - The idea behind the marker is to say that whenever two instances
    ///   of `TypedEncoded` share a marker they are guaranteed to also have
    ///   a common (but unknown) `type_id` so they can decode to the same
    ///   original type and thus we can allow to interoperate on them.
    ///
    /// # Example
    ///
    /// The `TestEnv` might use one abstract marker for every
    /// of the fundamental FRAME types: `Balance`, `AccountId`, `Hash`, etc.
    /// With this and the explicit guarantee that two instances of `TypedEncoded`
    /// with the same abstract marker also share the same (unknown) `type_id`
    /// it is possible to allow them to interoperate.
    marker: PhantomData<fn() -> T>,
}

/// Errors that may be encountered upon operating on typed encoded instances.
#[derive(Debug, From, PartialEq, Eq)]
pub enum TypedEncodedError {
    /// Error upon decoding.
    Decode(scale::Error),
    /// When operating on instances with different types.
    #[from(ignore)]
    DifferentTypes {
        lhs: core::any::TypeId,
        rhs: core::any::TypeId,
    },
    /// When an already initialized instance is about to be initialized.
    #[from(ignore)]
    AlreadyInitialized {
        initialized_id: core::any::TypeId,
        new_id: core::any::TypeId,
    },
    /// When operating on still uninitialized types.
    #[from(ignore)]
    StillUninitialized,
}

impl From<TypedEncodedError> for Error {
    fn from(typed_encoded_error: TypedEncodedError) -> Self {
        Error::OffChain(OffChainError::TypedEncoded(typed_encoded_error))
    }
}

/// The result type for typed encoded operations.
pub type Result<T> = core::result::Result<T, TypedEncodedError>;

impl<M> Default for TypedEncoded<M> {
    /// Creates an uninitialized instance.
    ///
    /// # Note
    ///
    /// The resulting instance can be properly initialized at a later point
    /// using a call to [`TypedEncoded::try_initialize`].
    fn default() -> Self {
        Self {
            encoded: Vec::new(),
            type_id: None,
            marker: Default::default(),
        }
    }
}

impl<M> TypedEncoded<M> {
    /// Creates a new uninitialized instance.
    pub fn uninitialized() -> Self {
        Self {
            encoded: Vec::new(),
            type_id: None,
            marker: Default::default(),
        }
    }

    /// Creates a new typed-encoded initialized by `value` of type `T`.
    pub fn new<T>(value: &T) -> Self
    where
        T: scale::Encode + 'static,
    {
        Self {
            encoded: value.encode(),
            type_id: Some(core::any::TypeId::of::<T>()),
            marker: Default::default(),
        }
    }

    /// Initializes `self` with a given encodable value.
    ///
    /// # Errors
    ///
    /// If `self` has already been initialized or is an initialized instance.
    pub fn try_initialize<T>(&mut self, value: &T) -> Result<()>
    where
        T: scale::Encode + 'static,
    {
        if let Some(id) = self.type_id {
            return Err(TypedEncodedError::AlreadyInitialized {
                initialized_id: id,
                new_id: core::any::TypeId::of::<T>(),
            })
        }
        value.encode_to(&mut self.encoded);
        self.type_id = Some(core::any::TypeId::of::<T>());
        Ok(())
    }

    /// Returns the encoded bytes representation.
    ///
    /// # Errors
    ///
    /// If the instance is still uninitialized.
    pub fn encoded_bytes(&self) -> Result<&[u8]> {
        if self.type_id.is_none() {
            return Err(TypedEncodedError::StillUninitialized)
        }
        Ok(&self.encoded[..])
    }

    /// Returns a mutable reference to the encoded bytes representation.
    ///
    /// # Errors
    ///
    /// If the instance is still uninitialized.
    pub fn encoded_bytes_mut(&mut self) -> Result<&mut [u8]> {
        if self.type_id.is_none() {
            return Err(TypedEncodedError::StillUninitialized)
        }
        Ok(&mut self.encoded[..])
    }

    /// Returns the type ID if the instance has already been initialized.
    ///
    /// # Errors
    ///
    /// Returns an appropriate error in case the instance is uninitialized.
    fn type_id(&self) -> Result<core::any::TypeId> {
        match self.type_id {
            Some(type_id) => Ok(type_id),
            None => Err(TypedEncodedError::StillUninitialized),
        }
    }

    /// Returns `Ok` if both types are encoded with the same type.
    fn check_matching_types(&self, other: &Self) -> Result<()> {
        let id_lhs = self.type_id()?;
        let id_rhs = other.type_id()?;
        if id_lhs != id_rhs {
            return Err(TypedEncodedError::DifferentTypes {
                lhs: id_lhs,
                rhs: id_rhs,
            })
        }
        Ok(())
    }

    /// Returns `Ok` if `T` is the type represented by the typed encoded instance.
    fn check_enforced_type<T>(&self) -> Result<()>
    where
        T: 'static,
    {
        let id_self = self.type_id()?;
        let id_enforced = core::any::TypeId::of::<T>();
        if core::any::TypeId::of::<T>() != id_self {
            return Err(TypedEncodedError::DifferentTypes {
                lhs: id_self,
                rhs: id_enforced,
            })
        }
        Ok(())
    }

    /// Decodes the instance.
    ///
    /// # Note
    ///
    /// This effectively creates a clone of the encoded value.
    pub fn decode<T>(&self) -> Result<T>
    where
        T: scale::Decode + 'static,
    {
        self.check_enforced_type::<T>()?;
        <T as scale::Decode>::decode(&mut &self.encoded[..]).map_err(Into::into)
    }

    /// Assigns the given `T` to `self`.
    pub fn assign<T>(&mut self, value: &T) -> Result<()>
    where
        T: scale::Encode + 'static,
    {
        self.check_enforced_type::<T>()?;
        self.encoded.clear();
        value.encode_to(&mut self.encoded);
        self.type_id = Some(core::any::TypeId::of::<T>());
        Ok(())
    }

    /// Evaluates the given closure on the given typed encoded instances.
    pub fn eval<T, F, R>(&self, other: &Self, f: F) -> Result<R>
    where
        T: scale::Decode + 'static,
        F: FnOnce(&T, &T) -> R,
    {
        Self::check_matching_types(self, other)?;
        let decoded_self = self.decode::<T>()?;
        let decoded_other = other.decode::<T>()?;
        Ok(f(&decoded_self, &decoded_other))
    }

    /// Evaluates the given closure on the given typed decoded instances
    /// and writes back the result into the typed encoded instance.
    pub fn eval_mut<T, F, R>(&mut self, other: &Self, f: F) -> Result<R>
    where
        T: scale::Decode + scale::Encode + 'static,
        F: FnOnce(&mut T, &T) -> R,
    {
        Self::check_matching_types(self, other)?;
        let mut decoded_self = self.decode::<T>()?;
        let decoded_other = other.decode::<T>()?;
        let result = f(&mut decoded_self, &decoded_other);
        self.encoded.clear();
        scale::Encode::encode_to(&decoded_self, &mut self.encoded);
        Ok(result)
    }

    /// Returns `true` if both instances are of type `T` and are equal.
    ///
    /// # Note
    ///
    /// The equality check is performed on decoded instances.
    pub fn eq<T>(&self, other: &Self) -> Result<bool>
    where
        T: PartialEq + scale::Decode + 'static,
    {
        self.eval::<T, _, _>(other, |lhs, rhs| core::cmp::PartialEq::eq(lhs, rhs))
    }

    /// Returns order relation if both instances are of type `T`.
    ///
    /// # Note
    ///
    /// The order relation is performed on the decoded instances.
    pub fn cmp<T>(&self, other: &Self) -> Result<Ordering>
    where
        T: PartialOrd + Ord + scale::Decode + 'static,
    {
        self.eval::<T, _, _>(other, |lhs, rhs| core::cmp::Ord::cmp(lhs, rhs))
    }

    /// Computes the hash of the decoded typed instance if types match.
    pub fn hash<T, H>(&self, state: &mut H) -> Result<()>
    where
        T: scale::Decode + Hash + 'static,
        H: Hasher,
    {
        self.decode::<T>()?.hash(state);
        Ok(())
    }
}

impl<T> PartialEq for TypedEncoded<T> {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id && self.encoded == other.encoded
    }
}

impl<T> Eq for TypedEncoded<T> {}

impl<T> PartialOrd for TypedEncoded<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<T> Ord for TypedEncoded<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.encoded.cmp(&other.encoded)
    }
}
