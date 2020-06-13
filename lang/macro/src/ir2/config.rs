// Copyright 2018-2020 Parity Technologies (UK) Ltd.
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

/// The ink! configuration.
pub struct Config {
    /// The version of the ink! smart contract.
    version: InkVersion,
    /// If `true` enables the dynamic storage allocator
    /// facilities and code generation of the ink! smart
    /// contract. Does incure some overhead. The default is
    /// `true`.
    storage_alloc: Option<bool>,
    /// If `true` compiles this ink! smart contract always as
    /// if it was a dependency of another smart contract.
    /// This configuration is mainly needed for testing and
    /// the default is `false`.
    as_dependency: Option<bool>,
    /// The environmental types definition.
    ///
    /// This must be a type that implements `ink_core::env::EnvTypes` and can
    /// be used to change the underlying environmental types of an ink! smart
    /// contract.
    env_types: Option<EnvTypes>,
}

impl Config {
    /// Returns the environmental types definition if specified.
    /// Otherwise returns the default environmental types definition provided
    /// by ink!.
    pub fn env_types(&self) -> EnvTypes {
        self.env_types
            .as_ref()
            .cloned()
            .unwrap_or_default()
    }

    /// Returns `true` if the dynamic storage allocator facilities are enabled
    /// for the ink! smart contract, `false` otherwise.
    ///
    /// If nothing has been specified returns the default which is `true`.
    pub fn is_storage_allocator_enabled(&self) -> bool {
        self.storage_alloc.unwrap_or(true)
    }

    /// Return `true` if this ink! smart contract shall always be compiled as
    /// if it was a dependency of another smart contract, returns `false`
    /// otherwise.
    ///
    /// If nothing has been specified returns the default which is `false`.
    pub fn is_compile_as_dependency_enabled(&self) -> bool {
        self.as_dependency.unwrap_or(false)
    }
}

/// The ink! version triple given as the configuration.
pub struct InkVersion {
    pub major: usize,
    pub minor: usize,
    pub path: usize,
}

/// The environmental types definition.
#[derive(Clone)]
pub struct EnvTypes {
    /// The underlying Rust type.
    env_types: syn::Type,
}

impl Default for EnvTypes {
    fn default() -> Self {
        Self {
            env_types: syn::parse_quote! { ::ink_core::env::DefaultEnvTypes },
        }
    }
}
