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

use core::{
    cell::UnsafeCell,
    fmt,
    fmt::Debug,
    ptr::NonNull,
};

/// A cache for a `T` that allow to mutate the inner `T` through `&self`.
///
/// Internally this is a thin wrapper around an `UnsafeCell<T>`.
/// The main difference to `UnsafeCell` is that this type provides an out of the
/// box API to safely access the inner `T` as well for single threaded contexts.
pub struct CacheCell<T: ?Sized> {
    /// The inner value that is allowed to be mutated in shared contexts.
    inner: UnsafeCell<T>,
}

impl<T> CacheCell<T> {
    /// Creates a new cache cell from the given value.
    pub fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    /// Returns the inner value.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }
}

impl<T> Debug for CacheCell<T>
where
    T: ?Sized + Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <T as Debug>::fmt(self.as_inner(), f)
    }
}

impl<T> From<T> for CacheCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Default for CacheCell<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(<T as Default>::default())
    }
}

impl<T> CacheCell<T>
where
    T: ?Sized,
{
    /// Returns a shared reference to the inner value.
    pub fn as_inner(&self) -> &T {
        // SAFETY: This is safe since we are returning a shared reference back
        //         to the caller while this method itself accesses `self` as
        //         shared reference.
        unsafe { &*self.inner.get() }
    }

    /// Returns an exclusive reference to the inner value.
    pub fn as_inner_mut(&mut self) -> &mut T {
        // SAFETY: This is safe since we are returning the exclusive reference
        //         of the inner value through the `get_mut` API which itself
        //         requires exclusive reference access to the wrapping `self`
        //         disallowing aliasing exclusive references.
        unsafe { &mut *self.inner.get() }
    }

    /// Returns a mutable pointer to the inner value.
    pub fn get_ptr(&self) -> NonNull<T> {
        // SAFETY: The inner `T` of the internal `UnsafeCell` exists and thus
        //         the pointer that we get returned to it via `UnsafeCell::get`
        //         is never going to be `null`.
        unsafe { NonNull::new_unchecked(self.inner.get()) }
    }
}
