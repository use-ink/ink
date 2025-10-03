// Copyright (C) Use Ink (UK) Ltd.
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

use std::fmt;

/// Dummy error type for sandbox_client
#[derive(Debug, thiserror::Error)]
pub struct SandboxErr {
    msg: String,
}

impl SandboxErr {
    /// Create a new `SandboxErr` with the given message.
    #[allow(dead_code)]
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl From<String> for SandboxErr {
    fn from(msg: String) -> Self {
        Self { msg }
    }
}

impl fmt::Display for SandboxErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SandboxErr: {}", self.msg)
    }
}
