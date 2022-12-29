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

//! Contract's stuff related to the environment.

/// Entrypoint of the contract to execute constructors or messages.
pub trait Entrypoint {
    /// Entrypoint to run a constructor for the contract. It deploys the contract to the environment.
    fn deploy();

    /// Entrypoint to run a message of the contract.
    fn call();
}
