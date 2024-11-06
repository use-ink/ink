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

use core::marker::PhantomData;

/// Can be used to check equality of types.
///
/// # Example
///
/// This code compiles:
///
/// ```
/// # use ink::codegen::IsSameType;
/// const _: IsSameType<i32> = IsSameType::<i32>::new();
/// ```
///
/// While this code does not:
///
/// ```compile_fail
/// # use ink::codegen::IsSameType;
/// const _: IsSameType<i32> = IsSameType::<i64>::new();
/// ```
pub struct IsSameType<T> {
    _marker: PhantomData<T>,
}

impl<T> IsSameType<T> {
    /// Creates a new const instance.
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Default for IsSameType<T> {
    fn default() -> Self {
        Self::new()
    }
}
