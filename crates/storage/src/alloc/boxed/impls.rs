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

use super::Box as StorageBox;
use crate::traits::{
    clear_spread_root,
    SpreadLayout,
};

impl<T> Drop for StorageBox<T>
where
    T: SpreadLayout,
{
    fn drop(&mut self) {
        clear_spread_root::<T>(self, &self.allocation.key());
        crate::alloc::free(self.allocation);
    }
}

impl<T> core::cmp::PartialEq for StorageBox<T>
where
    T: PartialEq + SpreadLayout,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(StorageBox::get(self), StorageBox::get(other))
    }
}

impl<T> core::cmp::Eq for StorageBox<T> where T: Eq + SpreadLayout {}

impl<T> core::cmp::PartialOrd for StorageBox<T>
where
    T: PartialOrd + SpreadLayout,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(StorageBox::get(self), StorageBox::get(other))
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(StorageBox::get(self), StorageBox::get(other))
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(StorageBox::get(self), StorageBox::get(other))
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(StorageBox::get(self), StorageBox::get(other))
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(StorageBox::get(self), StorageBox::get(other))
    }
}

impl<T> core::cmp::Ord for StorageBox<T>
where
    T: core::cmp::Ord + SpreadLayout,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(StorageBox::get(self), StorageBox::get(other))
    }
}

impl<T> core::fmt::Display for StorageBox<T>
where
    T: core::fmt::Display + SpreadLayout,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(StorageBox::get(self), f)
    }
}

impl<T> core::hash::Hash for StorageBox<T>
where
    T: core::hash::Hash + SpreadLayout,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        StorageBox::get(self).hash(state)
    }
}

impl<T> core::convert::AsRef<T> for StorageBox<T>
where
    T: SpreadLayout,
{
    fn as_ref(&self) -> &T {
        StorageBox::get(self)
    }
}

impl<T> core::convert::AsMut<T> for StorageBox<T>
where
    T: SpreadLayout,
{
    fn as_mut(&mut self) -> &mut T {
        StorageBox::get_mut(self)
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for StorageBox<T>
where
    T: SpreadLayout,
{
    fn borrow(&self) -> &T {
        StorageBox::get(self)
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for StorageBox<T>
where
    T: SpreadLayout,
{
    fn borrow_mut(&mut self) -> &mut T {
        StorageBox::get_mut(self)
    }
}

impl<T> core::ops::Deref for StorageBox<T>
where
    T: SpreadLayout,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        StorageBox::get(self)
    }
}

impl<T> core::ops::DerefMut for StorageBox<T>
where
    T: SpreadLayout,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        StorageBox::get_mut(self)
    }
}
