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

//! Utility definitions used for environmental access.

use crate::memory::vec::Vec;
use smallvec::{
    Array,
    SmallVec,
};

/// Buffers that allow to reset themselves.
///
/// # Note
///
/// Reset buffers are guaranteed to have a `len` of 0.
pub trait Reset {
    /// Resets the buffer.
    fn reset(&mut self);
}

impl<T> Reset for Vec<T> {
    fn reset(&mut self) {
        self.clear()
    }
}

impl<T> Reset for SmallVec<T>
where
    T: Array,
{
    fn reset(&mut self) {
        self.clear()
    }
}

/// Buffers that allow to enlarge themselves to the specified minimum length.
pub trait EnlargeTo {
    /// Enlarge `self` to fit at least `new_size` elements.
    ///
    /// # Note
    ///
    /// This should be implemented as a no-op if `self` is already big enough.
    fn enlarge_to(&mut self, new_size: usize);
}

impl<T> EnlargeTo for Vec<T>
where
    T: Default + Clone,
{
    fn enlarge_to(&mut self, new_size: usize) {
        if self.len() < new_size {
            self.resize(new_size, Default::default())
        }
    }
}

impl<T> EnlargeTo for SmallVec<T>
where
    T: Array,
    <T as smallvec::Array>::Item: Default + Clone,
{
    fn enlarge_to(&mut self, new_size: usize) {
        if self.len() < new_size {
            self.resize(new_size, Default::default())
        }
    }
}
