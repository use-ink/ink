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

//! Utilities, types and abstractions common to call and instantiation routines.

use core::marker::PhantomData;

use ink_primitives::{
    abi::{
        AbiDecodeWith,
        Ink,
        Sol,
    },
    MessageResult,
    SolDecode,
};
use scale::{
    Decode,
    DecodeAll,
};

/// Represents a return type.
///
/// Used as a marker type to define the return type of an ink! message in call builders.
#[derive(Debug)]
pub struct ReturnType<T>(PhantomData<fn() -> T>);

impl<T> Clone for ReturnType<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ReturnType<T> {}

impl<T> Default for ReturnType<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

/// A parameter that has been set to some value.
#[derive(Debug, Copy, Clone)]
pub struct Set<T>(pub T);

impl<T> Set<T> {
    /// Returns the set value.
    #[inline]
    pub fn value(self) -> T {
        self.0
    }
}

/// A parameter that has not been set, yet.
#[derive(Debug)]
pub struct Unset<T>(PhantomData<fn() -> T>);

impl<T> Clone for Unset<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Unset<T> {}

impl<T> Default for Unset<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Implemented by [`Set`] and [`Unset`] in order to unwrap their value.
///
/// This is useful in case the use-site does not know if it is working with
/// a set or an unset value generically unwrap it using a closure for fallback.
pub trait Unwrap {
    /// The output type of the `unwrap_or_else` operation.
    type Output;

    /// Returns the set value or evaluates the given closure.
    fn unwrap_or_else<F>(self, f: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output;
}

impl<T> Unwrap for Unset<T> {
    type Output = T;

    #[inline]
    fn unwrap_or_else<F>(self, f: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output,
    {
        f()
    }
}

impl<T> Unwrap for Set<T> {
    type Output = T;

    #[inline]
    fn unwrap_or_else<F>(self, _: F) -> Self::Output
    where
        F: FnOnce() -> Self::Output,
    {
        self.value()
    }
}

/// A trait for decoding the output of a message based on different ABIs.
/// This is necessary as contracts with different ABIs have different return types.
/// For example, Solidity contracts return the output directly without `MessageResult`.
pub trait DecodeMessageResult<Abi>: Sized {
    /// Decodes the output of a message call, requiring the output
    /// to be wrapped with `MessageResult` (if not included in the output).
    fn decode_output(buffer: &[u8]) -> crate::Result<MessageResult<Self>>;
}

impl<R> DecodeMessageResult<Ink> for R
where
    R: Decode,
    MessageResult<R>: Decode,
{
    fn decode_output(mut buffer: &[u8]) -> crate::Result<MessageResult<Self>> {
        let decoded = MessageResult::<R>::decode_all(&mut buffer)?;
        Ok(decoded)
    }
}

impl<R> DecodeMessageResult<Sol> for R
where
    R: SolDecode,
{
    fn decode_output(buffer: &[u8]) -> crate::Result<MessageResult<Self>> {
        // Solidity ABI Encoded contracts return the data without
        // `MessageResult`.
        let decoded = R::decode_with(buffer)?;
        Ok(Ok(decoded))
    }
}
