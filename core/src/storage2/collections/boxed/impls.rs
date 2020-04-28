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

use super::Box as StorageBox;
use crate::storage2::{
    traits2::SpreadLayout,
    ClearForward,
    KeyPtr,
    PullForward,
    StorageFootprint,
};

impl<T> Drop for StorageBox<T>
where
    T: SpreadLayout,
    T: ClearForward + StorageFootprint,
{
    fn drop(&mut self) {
        ClearForward::clear_forward(
            &self.value,
            &mut KeyPtr::from(self.allocation.key()),
        );
        crate::storage2::alloc::free(self.allocation);
    }
}

impl<T> core::cmp::PartialEq for StorageBox<T>
where
    T: SpreadLayout,
    T: PartialEq + ClearForward + StorageFootprint + PullForward,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(StorageBox::get(self), StorageBox::get(other))
    }
}

impl<T> core::cmp::Eq for StorageBox<T>
where
    T: SpreadLayout,
    T: Eq + ClearForward + StorageFootprint + PullForward,
{
}

impl<T> core::cmp::PartialOrd for StorageBox<T>
where
    T: SpreadLayout,
    T: PartialOrd + ClearForward + StorageFootprint + PullForward,
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
    T: SpreadLayout,
    T: core::cmp::Ord + ClearForward + StorageFootprint + PullForward,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(StorageBox::get(self), StorageBox::get(other))
    }
}

impl<T> core::fmt::Display for StorageBox<T>
where
    T: SpreadLayout,
    T: core::fmt::Display + ClearForward + StorageFootprint + PullForward,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(StorageBox::get(self), f)
    }
}

impl<T> core::hash::Hash for StorageBox<T>
where
    T: SpreadLayout,
    T: core::hash::Hash + ClearForward + StorageFootprint + PullForward,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        StorageBox::get(self).hash(state)
    }
}

impl<T> core::convert::AsRef<T> for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn as_ref(&self) -> &T {
        StorageBox::get(self)
    }
}

impl<T> core::convert::AsMut<T> for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn as_mut(&mut self) -> &mut T {
        StorageBox::get_mut(self)
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn borrow(&self) -> &T {
        StorageBox::get(self)
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn borrow_mut(&mut self) -> &mut T {
        StorageBox::get_mut(self)
    }
}

impl<T> core::ops::Deref for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        StorageBox::get(self)
    }
}

impl<T> core::ops::DerefMut for StorageBox<T>
where
    T: SpreadLayout,
    T: StorageFootprint + ClearForward + PullForward,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        StorageBox::get_mut(self)
    }
}
