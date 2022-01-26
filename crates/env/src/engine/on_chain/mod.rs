// Copyright 2018-2022 Parity Technologies (UK) Ltd.
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
        EncodeScope,
        ScopedBuffer,
        StaticBuffer,
    },
    ext::Error,
};
use super::OnInstance;

/// The buffer with static size of 16 kB for the input of the contract.
///
/// If input requires more than that they will fail.
///
/// Please note that this is still an implementation detail and
/// might change. Users should generally avoid storing too big values
/// into single storage entries.
struct InputBuffer {
    initialized: bool,
    buffer: StaticBuffer,
}

/// The on-chain environment.
pub struct EnvInstance {
    /// Encode & decode buffer with static size of 16 kB.
    ///
    /// If operations require more than that they will fail.
    /// This limit was found to be a sweet spot after running benchmarks
    /// on Substrate's storage and load performance.
    ///
    /// Please note that this is still an implementation detail and
    /// might change. Users should generally avoid storing too big values
    /// into single storage entries.
    buffer: StaticBuffer,

    /// Please note that this variable should be initialized only one time.
    /// After it should be used only for read only.
    input_buffer: InputBuffer,
}

impl<'a> OnInstance<'a> for EnvInstance {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&'a mut Self) -> R,
    {
        static mut INSTANCE: EnvInstance = EnvInstance {
            buffer: StaticBuffer::new(),
            input_buffer: InputBuffer {
                initialized: false,
                buffer: StaticBuffer::new(),
            },
        };
        f(unsafe { &mut INSTANCE })
    }
}
