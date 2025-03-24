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

use ink_prelude::{
    string::String,
    vec::Vec,
};
use paste::paste;

use crate::types::Address;

/// Analog of `From` that can be implemented for foreign types.
///
/// # Note
/// A primary motivation for this local "From" trait is that, even for a local type `T`,
/// sequences/collections of `T` (i.e. `[T; N]`, `Vec<T>`) are foreign types.
/// However, we need to convert such sequences/collections for the (transitively)
/// associated [`alloy_sol_types::SolType::RustType`] type for
/// [`crate::sol::SolTypeDecode`] to compose complex representations of Solidity ABI
/// types.
///
/// Ref: <https://doc.rust-lang.org/reference/items/implementations.html#trait-implementation-coherence>
///
/// # Note
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolFrom<T>: Sized + private::Sealed {
    /// Converts to this type from the input type.
    fn sol_from(value: T) -> Self;
}

macro_rules! impl_sol_from_reflexive {
    ($($ty: ty),+ $(,)*) => {
        $(
            impl SolFrom<$ty> for $ty {
                fn sol_from(value: $ty) -> Self {
                    value
                }
            }

            impl private::Sealed for $ty {}
        )*
    };
}

impl_sol_from_reflexive! {
    // signed ints
    i8, i16, i32, i64, i128,
    // unsigned ints
    u8, u16, u32, u64, u128,
    // bool
    bool,
    // string
    String,
    // unit
    (),
}

impl SolFrom<alloy_sol_types::private::Address> for Address {
    fn sol_from(value: alloy_sol_types::private::Address) -> Self {
        Address(value.into_array())
    }
}
impl private::Sealed for Address {}

impl SolFrom<alloy_sol_types::private::Bytes> for Vec<u8> {
    fn sol_from(value: alloy_sol_types::private::Bytes) -> Self {
        value.to_vec()
    }
}

impl<const N: usize> SolFrom<alloy_sol_types::private::FixedBytes<N>> for [u8; N] {
    fn sol_from(value: alloy_sol_types::private::FixedBytes<N>) -> Self {
        value.0
    }
}

impl<T, U: SolFrom<T>, const N: usize> SolFrom<[T; N]> for [U; N] {
    fn sol_from(value: [T; N]) -> Self {
        value.map(U::sol_from)
    }
}
impl<T, const N: usize> private::Sealed for [T; N] {}

impl<T, U: SolFrom<T>> SolFrom<Vec<T>> for Vec<U> {
    fn sol_from(value: Vec<T>) -> Self {
        value.into_iter().map(U::sol_from).collect()
    }
}
impl<T> private::Sealed for Vec<T> {}

macro_rules! impl_sol_from_tuple {
    ($(($($idx: literal),+)),* $(,)*) => {
        $(
            paste! {
                impl<$([<T $idx>]),+, $([<U $idx>]: SolFrom<[<T $idx>]>),+> SolFrom<( $([<T $idx>],)+ )> for ( $([<U $idx>],)+ ) {
                    fn sol_from(value: ( $([<T $idx>],)+ )) -> Self {
                        ( $([<U $idx>]::sol_from(value.$idx),)+ )
                    }
                }

                impl<$([<U $idx>]),+> private::Sealed for ( $([<U $idx>],)+ ) {}
            }
        )*
    };
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
impl_sol_from_tuple! {
    (0),
    (0, 1),
    (0, 1, 2),
    (0, 1, 2, 3),
    (0, 1, 2, 3, 4),
    (0, 1, 2, 3, 4, 5),
    (0, 1, 2, 3, 4, 5, 6),
    (0, 1, 2, 3, 4, 5, 6, 7),
    (0, 1, 2, 3, 4, 5, 6, 7, 8),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
    (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),
}

pub(super) mod private {
    /// Seals implementation of `SolFrom`.
    pub trait Sealed {}
}
