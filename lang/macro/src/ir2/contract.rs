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

use crate::ir2;

/// The contract inline module `mod`.
///
/// Contains all of the ink! smart contract definitions.
///
/// # Example
///
/// ```no_compile
/// #[ink::contract(version = "0.1.0")]
/// mod my_contract {
///     // ... definitions
/// }
/// ```
pub struct Contract {
    /// The parsed Rust inline module.
    ///
    /// Contains all Rust module items after parsing. Note that while parsing
    /// the ink! module all ink! specific items are moved out of this AST based
    /// representation.
    pub ast: ir2::Module,
    /// The specified ink! configuration.
    config: ir2::Config,
}

impl Contract {
    /// Returns the identifier of the ink! inline module definition.
    ///
    /// # Note
    ///
    /// The ink! inline module definition is the module that comprises the
    /// whole ink! smart contract, e.g.:
    ///
    /// ```no_compile
    /// #[ink::contract(version = "0.1.0")]
    /// mod my_contract {
    ///     // ... definitions
    /// }
    /// ```
    ///
    /// In the above case the `module_ident` is `my_contract`.
    pub fn module(&self) -> &ir2::Module {
        &self.ast
    }

    /// Returns the configuration of the ink! smart contract.
    ///
    /// # Note
    ///
    /// The configuration is given via the `#[ink::contract(config))]` attribute
    /// macro annotation itself within the `(config)` part. The only mandatory
    /// field there is `version`. Other fields include but are not limited to:
    ///
    /// - `types`: To specify `EnvTypes` different from the default environment
    ///            types.
    /// - `storage-alloc`: If `true` enables the dynamic storage allocator
    ///                    facilities and code generation of the ink! smart
    ///                    contract. Does incure some overhead. The default is
    ///                    `true`.
    /// - `as-dependency`: If `true` compiles this ink! smart contract always as
    ///                    if it was a dependency of another smart contract.
    ///                    This configuration is mainly needed for testing and
    ///                    the default is `false`.
    pub fn config(&self) -> &ir2::Config {
        &self.config
    }
}
