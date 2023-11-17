// Copyright (C) Parity Technologies (UK) Ltd.
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

use ast::{
    AttrStyle,
    Crate,
};
use clippy_utils::diagnostics::span_lint_and_help;
use if_chain::if_chain;
use rustc_ast as ast;
use rustc_lint::{
    EarlyContext,
    EarlyLintPass,
    LintContext,
};
use rustc_middle::lint::in_external_macro;
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::sym;

declare_lint! {
    /// ## What it does
    /// Checks if a contract is annotated with the `no_main` outer attribute.
    ///
    /// ## Why is this necessary?
    /// Contracts must be annotated with `no_main` outer attribute when compiled to WebAssembly for
    /// deploying on a blockchain.
    ///
    /// ## Example
    ///
    /// ```rust
    /// // Bad: Contract does not contain the `no_main` attribute, so it cannot be compiled to Wasm
    /// #![cfg_attr(not(feature = "std"), no_std)]
    ///	#[ink::contract]
    ///	mod my_contract { /* ... */ }
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// #![cfg_attr(not(feature = "std"), no_std, no_main)]
    ///	#[ink::contract]
    ///	mod my_contract { /* ... */ }
    /// ```
    pub NO_MAIN,
    Deny,
    "contract must be annotated with the `no_main` outer attribute"
}

declare_lint_pass!(NoMain => [NO_MAIN]);

/// Returns true if the target architecture is suitable to be executed on-chain
fn is_contract_build(cx: &EarlyContext<'_>) -> bool {
    matches!(
        cx.sess().target.llvm_target.to_string().as_str(),
        "wasm32-unknown-unknown" | "riscv32i-unknown-none-elf"
    )
}

impl EarlyLintPass for NoMain {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &Crate) {
        // Disable when building for e2e tests
        if !is_contract_build(cx) {
            return
        }

        // `no_main` is an `Inner` attribute of `#![cfg_attr(...)]`
        if krate.attrs.iter().all(|attr| {
            if_chain! {
            if !in_external_macro(cx.sess(), attr.span);
            if let AttrStyle::Inner = attr.style;
            if attr.has_name(sym::no_main);
            then { false } else { true }}
        }) {
            span_lint_and_help(
                cx,
                NO_MAIN,
                krate.spans.inner_span,
                "contract must be annotated with the `no_main` outer attribute",
                None,
                "consider annotating contract with `#![cfg_attr(not(feature = \"std\"), no_std, no_main)]` or `#[no_main]`"
            )
        }
    }
}
