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

mod ext;
mod impls;
mod retcode;

use ink_prelude::vec::Vec;
use super::Instance;

pub(crate) use self::retcode::RetCode;

/// The on-chain environment.
pub struct WasmEnv {
    /// Encode & decode buffer for potentially reusing required dynamic allocations.
    buffer: Vec<u8>,
}

pub enum Accessor {}

impl Instance for Accessor {
    type Engine = WasmEnv;

    fn run<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self::Engine) -> R
    {
        static mut INSTANCE: WasmEnv = WasmEnv { buffer: Vec::new() };
        f(unsafe { &mut INSTANCE })
    }
}
