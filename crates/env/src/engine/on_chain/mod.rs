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

mod buffer;
mod ext;
mod impls;

use self::{
    buffer::{
        ScopedBuffer,
        StaticBuffer,
    },
    ext::Error,
};
use super::OnInstance;

/// The on-chain environment.
pub struct EnvInstance {
    /// Encode & decode buffer with static size of 16kB.
    ///
    /// If operations require more than that they will fail.
    /// This limit was chosen after benchmarking Substrate storage
    /// storage and load performance and was found to be a sweet spot.
    ///
    /// Please note that this is still an implementation detail and
    /// might change. Users should generally avoid storing too big values
    /// into single storage entries.
    buffer: StaticBuffer,
}

impl OnInstance for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        static mut INSTANCE: EnvInstance = EnvInstance {
            buffer: StaticBuffer::new(),
        };
        f(unsafe { &mut INSTANCE })
    }
}
