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

/// Simplifies interaction with the host environment via `self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait Env {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the host environment with `self.env()` syntax.
    fn env(self) -> Self::EnvAccess;
}

/// Simplifies interaction with the host environment via `Self`.
///
/// # Note
///
/// This is generally implemented for storage structs that include
/// their environment in order to allow the different dispatch functions
/// to use it for returning the contract's output.
pub trait StaticEnv {
    /// The access wrapper.
    type EnvAccess;

    /// Accesses the host environment with `Self::env()` syntax.
    fn env() -> Self::EnvAccess;
}
