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

/// Errors that can be encountered upon interaction with the off-chain environment.
#[cfg(feature = "ink-experimental-engine")]
pub type Error = ink_engine::Error;

/// Result of interacting with the off-chain environment.
#[cfg(feature = "ink-experimental-engine")]
pub type Result<R> = ink_engine::Result<R>;

/// Errors that can be encountered upon interaction with the off-chain environment.
#[cfg(all(
    any(feature = "std", test, doc),
    not(feature = "ink-experimental-engine")
))]
pub type Error = ink_env_types::Error<crate::engine::off_chain::OffChainError>;

/// Result of interacting with the off-chain environment.
#[cfg(all(
    any(feature = "std", test, doc),
    not(feature = "ink-experimental-engine")
))]
pub type Result<R> = ink_env_types::Result<R, crate::engine::off_chain::OffChainError>;

/// Errors that can be encountered upon environmental interaction.
#[cfg(not(any(feature = "std", test, doc, feature = "ink-experimental-engine")))]
pub type Error = ink_env_types::Error;

/// Result of interacting with the environment.
#[cfg(not(any(feature = "std", test, doc, feature = "ink-experimental-engine")))]
pub type Result<R> = ink_env_types::Result<R>;
