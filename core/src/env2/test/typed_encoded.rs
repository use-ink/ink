// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
    byte_utils,
    env2::{
        DefaultSrmlTypes,
        call::{
            Selector,
            CallData,
        },
        property,
        test::Storage,
        utils::{
            EnlargeTo,
            Reset,
        },
        EnvTypes,
        GetProperty,
        SetProperty,
        types,
    },
};
use scale::{Codec, Encode, Decode};
use core::{
    marker::PhantomData,
    any::TypeId,
};

/// A wrapper around an encoded entity that only allows type safe accesses.
///
/// # Note
///
/// Checks are implemented at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedEncoded {
    /// The bytes of the encoded representation of the type.
    encoded: Vec<u8>,
    /// The unique identifier of the encoded type.
    type_id: TypeId,
}

/// Marker that indicates untypedness for an instance of `TypedEncoded`.
enum Untyped {}

impl Default for TypedEncoded {
    fn default() -> Self {
        Self {
            encoded: Vec::new(),
            type_id: TypeId::of::<Untyped>(),
        }
    }
}

impl TypedEncoded {
    /// Returns the encoded bytes.
    fn as_bytes(&self) -> &[u8] {
        &self.encoded
    }
}

/// Encountered when trying to decode a `TypedEncoded` as an invalid type.
#[derive(Debug, PartialEq, Eq)]
pub struct UnmatchingType;

impl TypedEncoded
{
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

impl TypedEncoded
{
    /// Converts the original typed entity into its encoded representation.
    pub fn from_origin<T>(value: &T) -> Self
    where
        T: scale::Encode + 'static,
    {
        Self {
            encoded: value.encode(),
            type_id: TypeId::of::<T>(),
        }
    }
}
