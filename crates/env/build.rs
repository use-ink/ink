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
    path::PathBuf,
};

use ink_build_script_utils::*;

/// Write the engine static buffer capacity constant to a file that is then
/// included in the env/src/engine/on_chain/buffer.rs file.
fn write_env_engine_static_buffer_capacity() {
    let capacity = env_engine_static_buffer_capacity();
    let out_dir =
        PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` must be set by cargo!"));
    let out_file = out_dir.join("env_engine_static_buffer_capacity.rs");
    write_file_if_changed(out_file, capacity);
}

fn main() {
    write_env_engine_static_buffer_capacity();
}
