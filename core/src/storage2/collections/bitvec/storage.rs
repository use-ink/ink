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

use super::{
    Bits256,
    Bitvec as StorageBitvec,
};
use crate::storage2::{
    KeyPtr,
    Pack,
    PullForward,
    PushForward,
    StorageFootprint,
    Vec as StorageVec,
};

impl StorageFootprint for StorageBitvec {
    const VALUE: u64 = 1 + <StorageVec<Pack<Bits256>> as StorageFootprint>::VALUE;
}

impl PullForward for StorageBitvec {
    fn pull_forward(ptr: &mut KeyPtr) -> Self {
        Self {
            len: PullForward::pull_forward(ptr),
            bits: PullForward::pull_forward(ptr),
        }
    }
}

impl PushForward for StorageBitvec {
    fn push_forward(&self, ptr: &mut KeyPtr) {
        self.len.push_forward(ptr);
        self.bits.push_forward(ptr);
    }
}
