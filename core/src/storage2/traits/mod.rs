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
        }
    };
}

#[rustfmt::skip]
macro_rules! forward_supported_array_lens_ty {
    ( $mac:ident ) => {
        const _: () = {
            #[rustfmt::skip]
            use typenum::{
                Z0,
                 P1,  P2,  P3,  P4,  P5,  P6,  P7,  P8,  P9, P10,
                P11, P12, P13, P14, P15, P16, P17, P18, P19, P20,
                P21, P22, P23, P24, P25, P26, P27, P28, P29, P30,
                P31, P32,
            };
            $mac! {
                ( 0,  Z0), ( 1,  P1), ( 2,  P2), ( 3,  P3), ( 4,  P4),
                ( 5,  P5), ( 6,  P6), ( 7,  P7), ( 8,  P8), ( 9,  P9),
                (10, P10), (11, P11), (12, P12), (13, P13), (14, P14),
                (15, P15), (16, P16), (17, P17), (18, P18), (19, P19),
                (20, P20), (21, P21), (22, P22), (23, P23), (24, P24),
                (25, P25), (26, P26), (27, P27), (28, P28), (29, P29),
                (30, P30), (31, P31), (32, P32),
            }
        };
    };
}

mod clear;
mod footprint;
mod pull;
mod push;

use ink_primitives::Key;

pub use self::{
    clear::{
        ClearAt,
        ClearForward,
    },
    footprint::{
        storage_footprint_u128,
        storage_footprint_u64,
        SaturatingStorage,
        StorageFootprint,
        StorageFootprintOf,
    },
    pull::{
        PullAt,
        PullForward,
    },
    push::{
        PushAt,
        PushForward,
    },
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
        T: StorageFootprint,
        <T as StorageFootprint>::Value: typenum::Unsigned,
    {
        let copy = self.key;
        self.key += <<T as StorageFootprint>::Value as typenum::Unsigned>::U128;
        copy
    }
}
