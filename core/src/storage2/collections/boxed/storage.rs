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
    lazy::Lazy,
    ClearForward,
    KeyPtr,
    PullForward,
    PushForward,
    StorageFootprint,
};
use ink_primitives::Key;

impl<T> StorageFootprint for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    /// A boxed entity always uses exactly 1 cell for its storage.
    ///
    /// The indirectly stored storage entity is not considered because the
    /// `StorageSize` is only concerned with inplace storage usage.
    const VALUE: u64 = 1;
}

impl<T> PullForward for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        let key = <Key as PullForward>::pull_forward(ptr);
        Self {
            key,
            value: Lazy::lazy(key),
        }
    }
}

impl<T> PushForward for StorageBox<T>
where
    T: ClearForward + PushForward + StorageFootprint,
{
    fn push_forward(&self, ptr: &mut KeyPtr) {
        PushForward::push_forward(&self.key, ptr);
        PushForward::push_forward(&self.value, &mut KeyPtr::from(self.key));
    }
}

impl<T> ClearForward for StorageBox<T>
where
    T: ClearForward + StorageFootprint,
{
    fn clear_forward(&self, ptr: &mut KeyPtr) {
        ClearForward::clear_forward(&self.key, ptr);
        ClearForward::clear_forward(&self.value, &mut KeyPtr::from(self.key));
    }
}
