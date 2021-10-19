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

//! Utilities, types and abstractions common to call and instantiation routines.

use core::marker::PhantomData;

use crate::Environment;

/// Represents a return type.
///
/// Used as a marker type to differentiate at compile-time between invoke and evaluate.
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

impl<T> scale::Encode for Set<T>
where
    T: scale::Encode,
{
    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        self.0.encode_to(output)
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

impl<T> scale::Encode for Unset<T>
where
    T: Default + scale::Encode,
{
    #[inline]
    fn encode_to<O: scale::Output + ?Sized>(&self, output: &mut O) {
        <T as Default>::default().encode_to(output)
    }
}

impl<T> scale::EncodeLike<T> for Unset<T> where T: Default + scale::Encode {}

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

/// Returns a balance encoder from the implementing type.
///
/// # Note
///
/// This trait is only implemented by `Set<&E::Balance>` and `Unset<E::Balance>`
/// where `E` is an ink! environment type that implements the `Environment` trait.
pub trait BalanceEncoder<E>
where
    E: Environment,
{
    /// The balance encoder type.
    type Output: scale::EncodeLike<E::Balance>;

    /// Returns a balance encoder.
    fn as_balance_encoder(&self) -> Self::Output;
}

impl<'a, E> BalanceEncoder<E> for Set<&'a E::Balance>
where
    E: Environment,
{
    type Output = &'a E::Balance;

    fn as_balance_encoder(&self) -> Self::Output {
        self.value()
    }
}

impl<E> BalanceEncoder<E> for Unset<E::Balance>
where
    E: Environment,
{
    type Output = Self;

    fn as_balance_encoder(&self) -> Self::Output {
        *self
    }
}
