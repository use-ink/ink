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

use crate::backend::{
    EnvBackend,
    TypedEnvBackend,
};
use cfg_if::cfg_if;

pub trait OnInstance: EnvBackend + TypedEnvBackend {
    fn on_instance<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

cfg_if! {
    if #[cfg(all(not(feature = "std"), target_arch = "wasm32"))] {
        mod on_chain;
        pub use self::on_chain::EnvInstance;
    } else if #[cfg(all(feature = "std", feature = "ink-experimental-engine"))] {
        pub mod experimental_off_chain;
        pub use experimental_off_chain as off_chain;
        pub use self::experimental_off_chain::EnvInstance;
    } else if #[cfg(feature = "std")] {
        pub mod off_chain;
        pub use self::off_chain::EnvInstance;
        pub use self::off_chain::{
            AccountError,
            TypedEncodedError,
        };
    } else {
        compile_error! {
            "ink! only support compilation as `std` or `no_std` + `wasm32-unknown`"
        }
    }
}
