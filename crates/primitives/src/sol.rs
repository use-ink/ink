// Copyright (C) ink! contributors.
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

//! Abstractions for implementing Solidity ABI encoding/decoding for arbitrary Rust types.

mod bytes;
mod from;
mod types;

#[cfg(test)]
mod tests;

use alloy_sol_types::SolType as AlloySolType;
use core::ops::Deref;
use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::borrow::Cow;

pub use bytes::AsSolBytes;
pub use types::SolType;

/// Maps an arbitrary Rust/ink! type to a Solidity ABI type representation for Solidity
/// ABI encoding/decoding.
///
/// # Note
/// Implementing this trait entails:
/// - Declaring the equivalent Solidity ABI type via the `SolCodec::SolType` associated
///   type. See the [docs for sealed `SolType` trait][SolType] for a table of Rust/ink!
///   primitive types mapped to their equivalent Solidity ABI type.
/// - Implementing two required methods that define how to convert between this type and
///   its equivalent Solidity ABI representation
///   - `from_sol_type`: for converting from `Self::SolType` to `Self` (used in decoding).
///   - `to_sol_type`: for converting (preferably via a borrow) from `&self` to
///     `&Self::SolType` (used in encoding).
///
/// # Example
///
/// ```
/// use core::convert::From;
/// use ink_prelude::borrow::Cow;
/// use ink_primitives::SolCodec;
///
/// // Example arbitrary type.
/// struct MyType {
///     size: u8,
///     status: bool,
/// }
///
/// // `SolCodec` implementation/mapping.
/// impl SolCodec for MyType {
///     type SolType = (u8, bool);
///
///     fn to_sol_type(&self) -> Cow<(u8, bool)> {
///         Cow::Owned((self.size, self.status))
///     }
///
///     fn from_sol_type(value: Self::SolType) -> Self {
///         Self {
///             size: value.0,
///             status: value.1,
///         }
///     }
/// }
/// ```
pub trait SolCodec {
    /// Equivalent Solidity ABI type representation.
    type SolType: SolType + Clone;

    /// Name of equivalent Solidity ABI type.
    const SOL_NAME: &'static str =
        <<Self::SolType as SolType>::AlloyType as AlloySolType>::SOL_NAME;

    /// Solidity ABI encode the value.
    fn encode(&self) -> Vec<u8> {
        <Self::SolType as SolType>::encode(self.to_sol_type().deref())
    }

    /// Solidity ABI decode into this type.
    fn decode(data: &[u8]) -> Result<Self, alloy_sol_types::Error>
    where
        Self: Sized,
    {
        <Self::SolType as SolType>::decode(data).map(Self::from_sol_type)
    }

    /// Converts from `Self` to `Self::SolType` via either a borrow (if possible), or
    /// a possibly expensive conversion otherwise.
    fn to_sol_type(&self) -> Cow<Self::SolType>;

    /// Converts to `Self` from `Self::SolType`.
    fn from_sol_type(value: Self::SolType) -> Self;
}

impl<T: SolCodec + Clone, const N: usize> SolCodec for [T; N] {
    type SolType = [T::SolType; N];

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(self.each_ref().map(|item| item.to_sol_type().into_owned()))
    }

    fn from_sol_type(value: Self::SolType) -> Self {
        value.map(<T as SolCodec>::from_sol_type)
    }
}

impl<T: SolCodec> SolCodec for Vec<T> {
    type SolType = Vec<T::SolType>;

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(
            self.iter()
                .map(|item| item.to_sol_type().into_owned())
                .collect(),
        )
    }

    fn from_sol_type(value: Self::SolType) -> Self {
        value
            .into_iter()
            .map(<T as SolCodec>::from_sol_type)
            .collect()
    }
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolCodec for Tuple {
    for_tuples!( type SolType = ( #( Tuple::SolType ),* ); );

    fn to_sol_type(&self) -> Cow<Self::SolType> {
        Cow::Owned(for_tuples! ( ( #( self.Tuple.to_sol_type().into_owned() ),* ) ))
    }

    #[allow(clippy::unused_unit)]
    fn from_sol_type(value: Self::SolType) -> Self {
        for_tuples!( ( #( Tuple::from_sol_type(value.Tuple) ),* ) );
    }
}
