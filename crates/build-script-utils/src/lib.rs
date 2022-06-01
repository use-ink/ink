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

use std::{
    env,
    fs,
    path::Path,
};

// Environment variables.
const ENV_ENGINE_STATIC_BUFFER_CAPACITY: &str = "ENV_ENGINE_STATIC_BUFFER_CAPACITY";

/// Configure the `StaticBuffer` capacity used in the `env` crate at build time
/// through the `ENV_ENGINE_STATIC_BUFFER_CAPACITY` environment variable.
/// If not explicitly configured, a default value of 16384 is used.
///
/// `StaticBuffer`: engine/on_chain/buffer.rs
pub fn env_engine_static_buffer_capacity() -> String {
    // Make sure that `build.rs` is called if the capacity configuration (env var) changes.
    println!(
        "cargo:rerun-if-env-changed={}",
        ENV_ENGINE_STATIC_BUFFER_CAPACITY
    );

    let capacity =
        env::var(ENV_ENGINE_STATIC_BUFFER_CAPACITY).unwrap_or_else(|_| "16384".into());
    let capacity: usize = capacity.parse().unwrap_or_else(|_| {
        panic!(
            "`{}` must be of type `usize`",
            ENV_ENGINE_STATIC_BUFFER_CAPACITY
        )
    });

    format!("const CONFIGURED_CAPACITY: usize = {};", capacity)
}

/// Write to the given `file` only if the `content` is different.
///
/// Taken from:
/// https://github.dev/paritytech/substrate/blob/4b48e8ec7dffcb599248040f4da5be3de3c09318/utils/wasm-builder/src/lib.rs#L151
pub fn write_file_if_changed(file: impl AsRef<Path>, content: impl AsRef<str>) {
    if fs::read_to_string(file.as_ref()).ok().as_deref() != Some(content.as_ref()) {
        fs::write(file.as_ref(), content.as_ref()).unwrap_or_else(|_| {
            panic!("Writing `{}` can not fail!", file.as_ref().display())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_engine_static_buffer_capacity_no_env() {
        env::remove_var(ENV_ENGINE_STATIC_BUFFER_CAPACITY);
        assert_eq!(
            env_engine_static_buffer_capacity(),
            "const CONFIGURED_CAPACITY: usize = 16384;"
        )
    }

    #[test]
    fn env_engine_static_buffer_capacity_valid_env() {
        env::set_var(ENV_ENGINE_STATIC_BUFFER_CAPACITY, "32768");
        assert_eq!(
            env_engine_static_buffer_capacity(),
            "const CONFIGURED_CAPACITY: usize = 32768;"
        )
    }

    #[test]
    #[should_panic]
    fn env_engine_static_buffer_capacity_invalid_env() {
        env::set_var(ENV_ENGINE_STATIC_BUFFER_CAPACITY, "abc");
        env_engine_static_buffer_capacity();
    }
}
