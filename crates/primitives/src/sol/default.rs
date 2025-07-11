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

use core::clone::Clone;

use impl_trait_for_tuples::impl_for_tuples;
use ink_prelude::{
    borrow::Cow,
    boxed::Box,
    string::String,
    vec::Vec,
};
use primitive_types::U256;

use crate::{
    sol::{
        bytes::SolBytesType,
        SolBytes,
        SolTypeEncode,
    },
    types::Address,
    H160,
};

/// Provides default representations of a [`SolTypeEncode`] implementing type.
///
/// # Note
///
/// This is useful for generating default encodings for cases where a Rust/ink! value
/// can't be mapped to any meaningful value for Solidity ABI encoding e.g.
/// encoding `Option::None<T>` where `T` could be a reference type.
///
/// Note that `core::default::Default` is NOT implemented for arrays (i.e. `[T; N]`) and
/// references (i.e. `&T` and `&mut T`).
///
/// This trait is sealed and cannot be implemented for types outside `ink_primitives`.
pub trait SolTypeDefault: SolTypeEncode {
    /// The default type representation.
    ///
    /// # Note
    ///
    /// Prefer an identity where possible for better performance.
    type DefaultType: SolTypeEncode;

    /// The default value.
    fn default() -> Self::DefaultType;

    /// Converts from this type to the Default type representation.
    ///
    /// # Note
    ///
    /// Prefer an identity where possible for better performance.
    fn to_default_type(self) -> Self::DefaultType;
}

macro_rules! impl_default_identity {
    ($($ty: ty => $value: expr),+ $(,)*) => {
        $(
            impl SolTypeDefault for $ty {
                type DefaultType = Self;

                fn default() -> Self::DefaultType {
                    $value
                }

                fn to_default_type(self) -> Self::DefaultType {
                    self
                }
            }
        )*
    };
}

impl_default_identity! {
    // bool
    bool => false,
    // signed integers
    i8 => 0i8,
    i16 => 0i16,
    i32 => 0i32,
    i64 => 0i64,
    i128 => 0i128,
    // unsigned integers
    u8 => 0u8,
    u16 => 0u16,
    u32 => 0u32,
    u64 => 0u64,
    u128 => 0u128,
    U256 => U256([0u64; 4]),
    // string
    &str => "",
    String => String::new(),
    Box<str> => Box::<str>::from(
        // SAFETY: empty string slice always produces valid UTF-8 bytes.
        unsafe { core::str::from_utf8_unchecked("".as_bytes()) },
    ),
    Cow<'_, str> => Cow::<str>::Borrowed(""),
    // address
    Address => H160([0u8; 20]),
}

// Implements `SolTypeDefault` for arrays.
impl<const N: usize, T> SolTypeDefault for [T; N]
where
    T: SolTypeDefault<DefaultType = T>,
{
    type DefaultType = [T::DefaultType; N];

    fn default() -> Self::DefaultType {
        core::array::from_fn(|_| T::default())
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// Implements `SolTypeDefault` for `Vec<T>`.
impl<T> SolTypeDefault for Vec<T>
where
    T: SolTypeDefault<DefaultType = T>,
{
    type DefaultType = Vec<T::DefaultType>;

    fn default() -> Self::DefaultType {
        Vec::new()
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// Implements `SolTypeDefault` for boxed slice (i.e. `Box<[T]>`).
impl<T> SolTypeDefault for Box<[T]>
where
    T: SolTypeDefault<DefaultType = T>,
{
    type DefaultType = Box<[T::DefaultType]>;

    fn default() -> Self::DefaultType {
        Box::from([T::default()])
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// Implements `SolTypeDefault` for `SolBytes<T>`.
impl<T> SolTypeDefault for SolBytes<T>
where
    T: SolBytesType,
{
    type DefaultType = Self;

    fn default() -> Self::DefaultType {
        SolBytes(<T as SolBytesType>::default())
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// We follow the Rust standard library's convention of implementing traits for tuples up
// to twelve items long.
// Ref: <https://doc.rust-lang.org/std/primitive.tuple.html#trait-implementations>
#[impl_for_tuples(12)]
impl SolTypeDefault for Tuple {
    for_tuples!( type DefaultType = ( #( Tuple::DefaultType ),* ); );

    #[allow(clippy::unused_unit)]
    fn default() -> Self::DefaultType {
        for_tuples!( ( #( Tuple::default() ),* ) )
    }

    #[allow(clippy::unused_unit)]
    fn to_default_type(self) -> Self::DefaultType {
        for_tuples!( ( #( self.Tuple.to_default_type() ),* ) )
    }
}

// Implements `SolTypeDefault` for reference types.
macro_rules! impl_default_refs {
    ($($ty: ty), +$(,)*) => {
        $(
            impl<'a, T> SolTypeDefault for $ty
            where
                T: SolTypeDefault<DefaultType = T> + Clone,
            {
                type DefaultType = Cow<'a, T>;

                fn default() -> Self::DefaultType {
                    Cow::Owned(T::default())
                }

                fn to_default_type(self) -> Self::DefaultType {
                    Cow::Borrowed(self)
                }
            }
        )*
    };
}

impl_default_refs! {
    &'a T, &'a mut T
}

// Implements `SolTypeEncode` for smart pointers.
impl<T> SolTypeDefault for Box<T>
where
    T: SolTypeDefault,
{
    type DefaultType = Box<T::DefaultType>;

    fn default() -> Self::DefaultType {
        Box::from(T::default())
    }

    fn to_default_type(self) -> Self::DefaultType {
        Box::from(T::to_default_type(*self))
    }
}

impl<T> SolTypeDefault for Cow<'_, T>
where
    T: SolTypeDefault<DefaultType = T> + Clone,
{
    type DefaultType = Self;

    fn default() -> Self::DefaultType {
        Cow::Owned(T::default())
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// Implements `SolTypeDefault` for `&mut str`.
impl<'a> SolTypeDefault for &'a mut str {
    type DefaultType = &'a str;

    fn default() -> Self::DefaultType {
        ""
    }

    fn to_default_type(self) -> Self::DefaultType {
        self
    }
}

// Implements `SolTypeEncode` for references to `[T]` DSTs.
macro_rules! impl_default_slice_refs {
    ($($ty: ty), +$(,)*) => {
        $(
            impl<'a, T> SolTypeDefault for $ty
            where
                T: SolTypeDefault<DefaultType = T> + Clone,
            {
                type DefaultType = Cow<'a, [T]>;

                fn default() -> Self::DefaultType {
                    Cow::Owned(Vec::new())
                }

                fn to_default_type(self) -> Self::DefaultType {
                    Cow::Borrowed(self)
                }
            }
        )*
    };
}

impl_default_slice_refs! {
    &'a [T], &'a mut [T]
}
