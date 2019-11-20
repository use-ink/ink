// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

use scale::{
    Decode,
    Encode,
};

/// Marks types as non-`Copy` and non-`Clone`.
///
/// # Note
///
/// - This marker type is zero-sized for all `T`.
/// - This serves as documentation for crate maintainers
///   as well as utility to enforce this behaviour for types
///   that are using it.
/// - Especially for `Cell` types it is important to make them
///   non-`Copy` and non-`Clone` since that would violate their
///   ownership guarantees over their contract storage slot.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct NonCloneMarker<T>(PhantomData<T>);

impl<T> Default for NonCloneMarker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
