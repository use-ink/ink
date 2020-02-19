// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

#[rustfmt::skip]
macro_rules! forward_supported_array_lens {
    ( $mac:ident ) => {
        $mac! {
            0,  1,  2,  3,  4,  5,  6,  7,  8,  9,
            10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
            20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
            30, 31, 32,
            64, 96, 128, 256, 512, 1024, 2048, 4096,
            8192, 16384,
        }
    };
}

/// Implemented by array of sizes of up to 32.
pub trait ArrayLenLessEquals32 {}
macro_rules! impl_array_len_less_equals_32_for {
    ( $($n:literal),* $(,)? ) => {
        $(
            impl<T> ArrayLenLessEquals32 for [T; $n] {}
        )*
    }
}
impl_array_len_less_equals_32_for!(
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 31, 32,
);

mod pull;
mod push;
mod storage_size;

use ink_primitives::Key;

pub use self::{
    pull::{
        PullAt,
        PullForward,
    },
    push::{
        PushAt,
        PushForward,
    },
    storage_size::StorageSize,
};

/// A key pointer.
///
/// Mainly used by [`PullForward`] and [`PushForward`] traits in order to provide
/// a streamlined interface for accessing the underlying [`Key`].
pub struct KeyPtr {
    /// The underlying key.
    key: Key,
}

impl From<Key> for KeyPtr {
    fn from(key: Key) -> Self {
        Self { key }
    }
}

impl KeyPtr {
    /// Returns the current `Key`.
    fn current(&self) -> Key {
        self.key
    }

    /// Advances the key by the given amount derive by `T` and its `StorageSize`
    /// and returns the next `Key` for usage by the storage element.
    pub fn next_for<T>(&mut self) -> Key
    where
        T: StorageSize,
    {
        let copy = self.key;
        self.key += <T as StorageSize>::SIZE;
        copy
    }
}
