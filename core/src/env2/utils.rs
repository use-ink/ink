// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

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
