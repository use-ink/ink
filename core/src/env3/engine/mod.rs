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

use cfg_if::cfg_if;
use crate::env3::backend::{TypedEnv, Env};

pub trait Instance {
    type Engine: Env + TypedEnv;

    fn run<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self::Engine) -> R;
}

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32-unknown"))] {
        mod off_chain;
        pub use self::off_chain::Accessor;
    } else if #[cfg(feature = "std")] {
        mod on_chain;
        pub use self::on_chain::Accessor;
    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}
