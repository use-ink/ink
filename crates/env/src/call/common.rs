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

//! Utilities, types and abstractions common to call and instantiation routines.

use core::marker::PhantomData;

/// Represents a return type.
///
/// Used as a marker type to define the return type of an ink! message in call builders.
#[derive(Debug)]
pub struct ReturnType<T>(PhantomData<fn() -> T>);

impl<T> Clone for ReturnType<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(Default::default())
    }
}

impl<T> Copy for ReturnType<T> {}

impl<T> Default for ReturnType<T> {
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

mod private {
    /// Seals the implementation of `ConstructorReturnType`.
    pub trait Sealed {}
}

/// Guards against using invalid contract initializer types.
///
/// # Note
///
/// Currently the only allowed types are `()` and `Result<(), E>`
/// where `E` is some unspecified error type.
/// If the contract initializer returns `Result::Err` the utility
/// method that is used to initialize an ink! smart contract will
/// revert the state of the contract instantiation.
pub trait InstantiateResult<C>: private::Sealed {
    /// Is `true` if `Self` is `Result<C, E>`.
    const IS_RESULT: bool = false;

    /// Reflects the output type of the dispatchable ink! constructor.
    type Output<O>;

    /// The error type of the constructor return type.
    type Error: scale::Decode;

    /// Construct a success value of the `Output` type.
    fn ok<O>(value: O) -> Self::Output<O>;

    /// Construct an error value of the `Output` type.
    fn err<O>(err: Self::Error) -> Self::Output<O>;
}

impl<T> private::Sealed for T {}

impl<C> InstantiateResult<C> for C {
    type Output<O> = O;
    type Error = ();

    fn ok<O>(value: O) -> Self::Output<O> {
        value
    }

    fn err<O>(_err: Self::Error) -> Self::Output<O> {
        // todo!
        unreachable!()
    }
}

impl<C, E> InstantiateResult<C> for Result<C, E>
where
    E: scale::Decode,
{
    const IS_RESULT: bool = true;

    type Output<O> = Result<O, E>;
    type Error = E;

    fn ok<O>(value: O) -> Self::Output<O> {
        Ok(value)
    }

    fn err<O>(err: Self::Error) -> Self::Output<O> {
        Err(err)
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
        Self(Default::default())
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

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn instantiate_result_types() {
//         static_assertions::assert_type_eq_all!(
//             Result<(), u8>,
//             <Result<(), u8> as InstantiateResult<()>>
//         );
//     }
// }
