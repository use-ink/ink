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
    ClearForward,
    KeyPtr,
    PullForward,
    StorageFootprint,
};

impl<T> Drop for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn drop(&mut self) {
        ClearForward::clear_forward(&self.value, &mut KeyPtr::from(self.key));
    }
}

impl<T> core::cmp::PartialEq for StorageBox<T>
where
    T: PartialEq + ClearForward + StorageFootprint + PullForward,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(self.get(), other.get())
    }
}

impl<T> core::cmp::Eq for StorageBox<T> where
    T: Eq + ClearForward + StorageFootprint + PullForward
{
}

impl<T> core::cmp::PartialOrd for StorageBox<T>
where
    T: PartialOrd + ClearForward + StorageFootprint + PullForward,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(self.get(), other.get())
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(self.get(), other.get())
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(self.get(), other.get())
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(self.get(), other.get())
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(self.get(), other.get())
    }
}

impl<T> core::cmp::Ord for StorageBox<T>
where
    T: core::cmp::Ord + ClearForward + StorageFootprint + PullForward,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(self.get(), other.get())
    }
}

impl<T> core::fmt::Display for StorageBox<T>
where
    T: core::fmt::Display + ClearForward + StorageFootprint + PullForward,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(self.get(), f)
    }
}

impl<T> core::hash::Hash for StorageBox<T>
where
    T: core::hash::Hash + ClearForward + StorageFootprint + PullForward,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T> core::convert::AsRef<T> for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn as_ref(&self) -> &T {
        self.get()
    }
}

impl<T> core::convert::AsMut<T> for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn as_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> ink_prelude::borrow::Borrow<T> for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn borrow(&self) -> &T {
        self.get()
    }
}

impl<T> ink_prelude::borrow::BorrowMut<T> for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn borrow_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> core::ops::Deref for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> core::ops::DerefMut for StorageBox<T>
where
    T: StorageFootprint + ClearForward + PullForward,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
