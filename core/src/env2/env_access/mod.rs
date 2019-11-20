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

mod immutable;
mod mutable;

pub use self::{
    immutable::EnvAccess,
    mutable::{
        EmitEvent,
        EnvAccessMut,
    },
};

/// Allows to access the environment from `&EnvAccess` and `&mut EnvAccess`
/// respectively with different degree of efficiency.
pub trait AccessEnv {
    /// The environment definition.
    type Target;

    /// Access the environment.
    fn env(self) -> Self::Target;
}
