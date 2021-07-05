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

use crate::{
    ast,
    ir,
};
use core::convert::TryFrom;
use proc_macro2::TokenStream as TokenStream2;

/// An ink! contract definition consisting of the ink! configuration and module.
///
/// This is the root of any ink! smart contract definition. It contains every
/// information accessible to the ink! smart contract macros. It is also used
/// as the root source for the ink! code generation.
///
/// # Example
///
/// ```no_compile
/// #[ink::contract(/* optional ink! configurations */)]
/// mod my_contract {
///     /* ink! and Rust definitions */
/// }
/// ```
pub struct Contract {
    /// The parsed Rust inline module.
    ///
    /// Contains all Rust module items after parsing. Note that while parsing
    /// the ink! module all ink! specific items are moved out of this AST based
    /// representation.
    item: ir::ItemMod,
    /// The specified ink! configuration.
    config: ir::Config,
}

impl Contract {
    /// Creates a new ink! contract from the given ink! configuration and module
    /// token streams.
    ///
    /// The ink! macro should use this constructor in order to setup ink!.
    ///
    /// # Note
    ///
    /// - The `ink_config` token stream must properly decode into [`ir::Config`].
    /// - The `ink_module` token stream must properly decode into [`ir::ItemMod`].
    ///
    /// # Errors
    ///
    /// Returns an error if the provided token stream cannot be decoded properly
    /// into a valid ink! configuration or ink! module respectively.
    pub fn new(
        ink_config: TokenStream2,
        ink_module: TokenStream2,
    ) -> Result<Self, syn::Error> {
        let config = syn::parse2::<ast::AttributeArgs>(ink_config)?;
        let module = syn::parse2::<syn::ItemMod>(ink_module)?;
        let ink_config = ir::Config::try_from(config)?;
        let ink_module = ir::ItemMod::try_from(module)?;
        Ok(Self {
            item: ink_module,
            config: ink_config,
        })
    }

    /// Returns the ink! inline module definition.
    ///
    /// # Note
    ///
    /// The ink! inline module definition is the module that comprises the
    /// whole ink! smart contract, e.g.:
    ///
    /// ```no_compile
    /// #[ink::contract]
    /// mod my_contract {
    ///     // ... definitions
    /// }
    /// ```
    pub fn module(&self) -> &ir::ItemMod {
        &self.item
    }

    /// Returns the configuration of the ink! smart contract.
    ///
    /// # Note
    ///
    /// The configuration is given via the `#[ink::contract(config))]` attribute
    /// macro annotation itself within the `(config)` part. The available fields
    /// are the following:
    ///
    /// - `types`: To specify `Environment` different from the default environment
    ///            types.
    /// - `storage-alloc`: If `true` enables the dynamic storage allocator
    ///                    facilities and code generation of the ink! smart
    ///                    contract. Does incur some overhead. The default is
    ///                    `true`.
    /// - `as-dependency`: If `true` compiles this ink! smart contract always as
    ///                    if it was a dependency of another smart contract.
    ///                    This configuration is mainly needed for testing and
    ///                    the default is `false`.
    ///
    /// Note that we might add more configuration fields in the future if
    /// necessary.
    pub fn config(&self) -> &ir::Config {
        &self.config
    }
}
