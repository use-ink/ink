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

use ast::{
    AttrStyle,
    Crate,
};
use if_chain::if_chain;
use ink_linting_utils::clippy::diagnostics::span_lint_and_help;
use rustc_ast as ast;
use rustc_lint::{
    EarlyContext,
    EarlyLintPass,
    LintContext,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::sym;

declare_lint! {
    /// ## What it does
    /// Checks if a contract is annotated with the `no_main` inner attribute.
    ///
    /// ## Why is this necessary?
    /// Contracts must be annotated with `no_main` inner attribute when compiled for on-chain
    /// execution.
    ///
    /// ## Example
    ///
    /// ```rust
    /// // Bad: Contract does not contain the `no_main` attribute,
    /// // so it cannot be compiled to Wasm
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
    "contract must be annotated with the `no_main` inner attribute"
}

declare_lint_pass!(NoMain => [NO_MAIN]);

impl EarlyLintPass for NoMain {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &Crate) {
        // `no_main` is an `Inner` attribute of `#![cfg_attr(...)]`
        if krate.attrs.iter().all(|attr| {
            if_chain! {
            if !attr.span.in_external_macro(cx.sess().source_map());
            if let AttrStyle::Inner = attr.style;
            if attr.has_name(sym::no_main);
            then { false } else { true }}
        }) {
            span_lint_and_help(
                cx,
                NO_MAIN,
                krate.spans.inner_span,
                "contract must be annotated with the `no_main` inner attribute",
                None,
                "consider annotating contract with `#![cfg_attr(not(feature = \"std\"), no_std, no_main)]` or `#![no_main]`\n\
                for further information visit https://use.ink/linter/rules/no_main",
            )
        }
    }
}
