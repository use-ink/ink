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

use alloy_sol_types::{
    Word,
    abi::{
        Encoder,
        token::{
            DynSeqToken,
            FixedSeqToken,
            PackedSeqToken,
            Token,
            WordToken,
        },
    },
};
use ink_prelude::vec::Vec;

/// A Solidity ABI encodable representation for a type.
///
///
/// # Note
///
/// An analog of `Token` and `TokenSeq` with only encoding operations.
///
/// References:
///
/// - <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L54>
/// - <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L85>
//
// # Design Notes
//
// This trait allows us to encode using local abstractions, notably
// `TokenOrDefault`, `FixedSizeDefault` and `DynSizeDefault`, for which we can't implement
// `Token` nor `TokenSeq` because those are "sealed" traits in `alloy_sol_types`.
pub trait Encodable: private::Sealed {
    /// True for dynamic types.
    const DYNAMIC: bool;

    /// Number of words in the head.
    fn head_words(&self) -> usize;

    /// Number of words in the tail.
    fn tail_words(&self) -> usize;

    /// Total number of head and tails words.
    #[inline(always)]
    fn total_words(&self) -> usize {
        self.head_words() + self.tail_words()
    }

    /// Append head words to the encoder.
    fn head_append(&self, encoder: &mut Encoder);

    /// Append tail words to the encoder.
    fn tail_append(&self, encoder: &mut Encoder);

    /// Append both head and tail words to the encoder.
    fn encode(&self, encoder: &mut Encoder) {
        // Head is either the actual data (for fixed-sized types) or the offset (for
        // dynamic types).
        encoder.push_offset(Encodable::head_words(self));
        Encodable::head_append(self, encoder);
        if <Self as Encodable>::DYNAMIC {
            // Only dynamic types have tails, which contain the "actual data".
            encoder.bump_offset(Encodable::tail_words(self));
            Encodable::tail_append(self, encoder);
        }
        // Encoder implementation detail for tracking offsets.
        encoder.pop_offset();
    }
}

// NOTE: We use a macro instead of a generic implementation over `T: Token` because
// that would "conflict" with generic implementations over `T: Encodable`.
macro_rules! impl_encodable_for_token {
    ($([$($gen:tt)*] $ty: ty),+ $(,)*) => {
        $(
            impl<$($gen)*> Encodable for $ty {
                const DYNAMIC: bool = <$ty as Token>::DYNAMIC;

                fn head_words(&self) -> usize {
                    Token::head_words(self)
                }

                fn tail_words(&self) -> usize {
                    Token::tail_words(self)
                }

                fn head_append(&self, encoder: &mut Encoder) {
                    Token::head_append(self, encoder);
                }

                fn tail_append(&self, encoder: &mut Encoder) {
                    Token::tail_append(self, encoder);
                }
            }

            impl<$($gen)*> private::Sealed for $ty {}
        )+
    };
}

impl_encodable_for_token! {
    [] WordToken,
    [] PackedSeqToken<'_>,
    [T: for<'a> Token<'a>, const N: usize] FixedSeqToken<T, N>,
    [T: for<'a> Token<'a>] DynSeqToken<T>,
}

/// Either a `Token` based (i.e. "actual value") or "default value" (i.e.
/// `FixedSizeDefault` or `DynSizeDefault`) based representation.
#[derive(Debug)]
pub enum TokenOrDefault<T, D> {
    Token(T),
    Default(D),
}

/// A fixed-size type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedSizeDefault(usize);

impl FixedSizeDefault {
    /// Empty data.
    pub const EMPTY: Self = Self(0);

    /// A single word.
    pub const WORD: Self = Self(1);

    /// A fixed size number of words (e.g. for encoding `bytesN`).
    pub const fn words(size: usize) -> Self {
        Self(size)
    }
}

/// A dynamic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynSizeDefault;

impl Encodable for FixedSizeDefault {
    const DYNAMIC: bool = false;

    fn head_words(&self) -> usize {
        // All the data is in the head.
        self.0
    }

    fn tail_words(&self) -> usize {
        // No tail words, because all the data is in the head, or it's empty.
        0
    }

    fn head_append(&self, encoder: &mut Encoder) {
        match self.0 {
            0 => (),
            1 => {
                // Appends empty word.
                encoder.append_word(Word::from([0u8; 32]));
            }
            size => {
                // Appends empty words.
                // NOTE: Appending bytes directly would be more efficient but `Encoder`
                // doesn't currently have a public method for doing this.
                let mut counter = 0;
                while counter < size {
                    encoder.append_word(Word::from([0u8; 32]));
                    counter += 1;
                }
            }
        }
    }

    fn tail_append(&self, _: &mut Encoder) {}
}

impl private::Sealed for FixedSizeDefault {}

impl Encodable for DynSizeDefault {
    const DYNAMIC: bool = true;

    fn head_words(&self) -> usize {
        // offset.
        1
    }

    fn tail_words(&self) -> usize {
        // length (i.e. zero), no "actual data".
        1
    }

    fn head_append(&self, encoder: &mut Encoder) {
        // Appends offset.
        encoder.append_indirection();
    }

    fn tail_append(&self, encoder: &mut Encoder) {
        encoder.append_seq_len(0);
    }
}

impl private::Sealed for DynSizeDefault {}

impl<T, D> Encodable for TokenOrDefault<T, D>
where
    T: Encodable,
    D: Encodable,
{
    const DYNAMIC: bool = T::DYNAMIC;

    fn head_words(&self) -> usize {
        match self {
            TokenOrDefault::Token(token) => token.head_words(),
            TokenOrDefault::Default(default) => default.head_words(),
        }
    }

    fn tail_words(&self) -> usize {
        match self {
            TokenOrDefault::Token(token) => token.tail_words(),
            TokenOrDefault::Default(default) => default.tail_words(),
        }
    }

    fn head_append(&self, encoder: &mut Encoder) {
        match self {
            TokenOrDefault::Token(token) => token.head_append(encoder),
            TokenOrDefault::Default(default) => default.head_append(encoder),
        }
    }

    fn tail_append(&self, encoder: &mut Encoder) {
        match self {
            TokenOrDefault::Token(token) => token.tail_append(encoder),
            TokenOrDefault::Default(default) => default.tail_append(encoder),
        }
    }
}

impl<T, D> private::Sealed for TokenOrDefault<T, D> {}

// Analog of `FixedSeqToken` but with `T` bound being `Encodable` instead of `Token` and
// `TokenSeq`.
//
// Ref: <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L253>
impl<T, const N: usize> Encodable for [T; N]
where
    T: Encodable,
{
    const DYNAMIC: bool = T::DYNAMIC;

    fn head_words(&self) -> usize {
        if Self::DYNAMIC {
            // offset.
            1
        } else {
            // elements.
            self.iter().map(T::total_words).sum()
        }
    }

    fn tail_words(&self) -> usize {
        if Self::DYNAMIC {
            // elements.
            self.iter().map(T::total_words).sum()
        } else {
            0
        }
    }

    fn head_append(&self, encoder: &mut Encoder) {
        if Self::DYNAMIC {
            // Appends offset.
            encoder.append_indirection();
        } else {
            // Appends "actual data".
            for inner in self {
                inner.head_append(encoder);
            }
        }
    }

    fn tail_append(&self, encoder: &mut Encoder) {
        // Appends "actual data" to the tail for dynamic elements.
        if Self::DYNAMIC {
            encode_sequence(self, encoder);
        }
    }
}

impl<T, const N: usize> private::Sealed for [T; N] {}

// Analog of `DynSeqToken` but with `T` bound being `Encodable` instead of `Token` and
// `TokenSeq`.
//
// Ref: <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L366>
impl<T> Encodable for Vec<T>
where
    T: Encodable,
{
    const DYNAMIC: bool = true;

    fn head_words(&self) -> usize {
        // offset.
        1
    }

    fn tail_words(&self) -> usize {
        // length + elements.
        1 + self.iter().map(T::total_words).sum::<usize>()
    }

    fn head_append(&self, encoder: &mut Encoder) {
        // Adds offset.
        encoder.append_indirection();
    }

    fn tail_append(&self, encoder: &mut Encoder) {
        // Appends length.
        encoder.append_seq_len(self.len());

        // Appends "actual data".
        encode_sequence(self, encoder);
    }
}

impl<T> private::Sealed for Vec<T> {}

/// Identical to `TokenSeq::encode_sequence` implementations for `FixedSeqToken` and
/// `DynSeqToken` but with `T` bound being `Encodable` instead of `Token`.
///
/// References:
/// - <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L305>
/// - <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L409>
fn encode_sequence<T>(tokens: &[T], encoder: &mut Encoder)
where
    T: Encodable,
{
    encoder.push_offset(tokens.iter().map(T::head_words).sum());
    for inner in tokens {
        inner.head_append(encoder);
        encoder.bump_offset(inner.tail_words());
    }
    for inner in tokens {
        inner.tail_append(encoder);
    }
    encoder.pop_offset();
}

/// A Solidity ABI encodable representation of function parameters.
///
/// # Note
///
/// This trait is only implemented for tuples which also implement [`Encodable`].
pub trait EncodableParams: private::Sealed {
    /// Encode the function parameters into the encoder.
    fn encode_params(&self, encoder: &mut Encoder);
}

/// Generates `EncodableParams` implementation body for an n-ary tuple where n >= 1.
// NOTE: operation is a noop for unit (i.e. `()`).
macro_rules! impl_encodable_params {
    ($source: ident, $encoder: ident => ($($ty:ident),+$(,)*)) => {
        let ($($ty,)+) = $source;
        $encoder.push_offset(0 $( + $ty.head_words() )+);

        $(
            $ty.head_append($encoder);
            $encoder.bump_offset($ty.tail_words());
        )+

        $(
            $ty.tail_append($encoder);
        )+

        $encoder.pop_offset();
    };
}

/// Identical to tuple implementations for `T: Token` and `T: TokenSeq` but with
/// `T: Encodable` as the bound.
///
/// Ref: <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L521>
// `impl-trait-for-tuples` doesn't support using `||` as a separator needed for
// `Encodable::DYNAMIC`, so we fallback to a declarative macro.
macro_rules! impl_encodable {
    ($($ty:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($ty: Encodable,)+> Encodable for ($($ty,)+) {
            const DYNAMIC: bool = $(<$ty as Encodable>::DYNAMIC )||+;

            #[inline]
            fn head_words(&self) -> usize {
                if Self::DYNAMIC {
                    // offset
                    1
                } else {
                    // elements
                    let ($($ty,)+) = self;
                    0 $( + $ty.total_words() )+
                }
            }

            #[inline]
            fn tail_words(&self) -> usize {
                if Self::DYNAMIC {
                    // elements
                    let ($($ty,)+) = self;
                    0 $( + $ty.total_words() )+
                } else {
                    0
                }
            }

            #[inline]
            fn head_append(&self, encoder: &mut Encoder) {
                if Self::DYNAMIC {
                    encoder.append_indirection();
                } else {
                    let ($($ty,)+) = self;
                    $(
                        $ty.head_append(encoder);
                    )+
                }
            }

            #[inline]
            fn tail_append(&self, encoder: &mut Encoder) {
                if Self::DYNAMIC {
                    impl_encodable_params!(self, encoder => ($($ty,)+));
                }
            }
        }

        #[allow(non_snake_case)]
        impl<$($ty: Encodable,)+> EncodableParams for ($($ty,)+) {
            fn encode_params(&self, encoder: &mut Encoder) {
                impl_encodable_params!(self, encoder => ($($ty,)+));
            }
        }

        impl<$($ty: Encodable,)+> private::Sealed for ($($ty,)+) {}
    };
}

impl_all_tuples!(@nonempty impl_encodable);

// Identical to optimized `Token` and `TokenSeq` implementation for `()`, but for
// `Encodable`.
//
// Ref: <https://github.com/alloy-rs/core/blob/49b7bce463cce6e987a8fb9a987acbf4ec4297a6/crates/sol-types/src/abi/token.rs#L616>
impl Encodable for () {
    const DYNAMIC: bool = false;

    #[inline]
    fn head_words(&self) -> usize {
        0
    }

    #[inline]
    fn tail_words(&self) -> usize {
        0
    }

    #[inline]
    fn head_append(&self, _: &mut Encoder) {}

    #[inline]
    fn tail_append(&self, _: &mut Encoder) {}

    #[inline]
    fn encode(&self, _: &mut Encoder) {}
}

impl EncodableParams for () {
    fn encode_params(&self, _: &mut Encoder) {}
}

impl private::Sealed for () {}

pub(super) mod private {
    /// Seals implementations of `Encodable`.
    pub trait Sealed {}
}
