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

use clippy_utils::{
    diagnostics::span_lint_and_then,
    is_lint_allowed,
    match_def_path,
    source::snippet_opt,
};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::{
    self,
    def::{
        DefKind,
        Res,
    },
    def_id::DefId,
    Arm,
    AssocItemKind,
    ExprKind,
    Impl,
    ImplItemKind,
    ImplItemRef,
    Item,
    ItemKind,
    Node,
    PatKind,
    QPath,
};
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::ty::{
    self,
    Ty,
    TyKind,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};

declare_lint! {
    /// ## What it does
    ///
    /// The lint detects potentially unsafe uses of methods for which there are safer alternatives.
    ///
    /// ## Why is this bad?
    ///
    /// In some standard collections in ink!, there are two types of implementations: non-fallible
    /// (e.g. `get`) and fallible (e.g. `try_get`). Fallible alternatives are considered safer,
    /// as they perform additional checks for incorrect input parameters and return `Result::Err`
    /// when they are used improperly. On the other hand, non-fallible methods do not provide these
    /// checks and will panic on incorrect input, placing the responsibility on the user to
    /// implement these checks.
    ///
    /// For more context, see: [ink#1910](https://github.com/paritytech/ink/pull/1910).
    ///
    /// ## Example
    ///
    /// Consider the contract that has the following `Mapping` field:
    ///
    /// ```rust
    /// #[ink(storage)]
    /// pub struct Example { map: Mapping<String, AccountId> }
    /// ```
    ///
    /// The following usage of the non-fallible API is unsafe:
    ///
    /// ```rust
    /// pub fn test(&mut self, a: String, b: AccountId) {
    ///   // Bad: can panic on incorrect input
    ///   self.map.insert(a, &b);
    /// }
    /// ```
    ///
    /// It could be replaced with the fallible version of `Mapping::insert`:
    ///
    /// ```rust
    /// pub fn test(&mut self, a: String, b: AccountId) {
    ///   // Good: returns Result::Err on incorrect input
    ///   self.map.try_insert(a, &b)?;
    /// }
    /// ```
    ///
    /// Otherwise, the user could explicitly check the encoded size of the argument in their code:
    ///
    /// ```rust
    /// pub fn test(&mut self, a: String, b: AccountId) {
    ///   // Good: explicitly checked encoded size of the input
    ///   if String::encoded_size(&a) < ink_env::BUFFER_SIZE {
    ///     self.map.insert(a, &b);
    ///   }
    /// }
    /// ```
    pub NON_FALLIBLE_API,
    Warn,
    "using non-fallible API"
}

declare_lint_pass!(NonFallibleAPI => [NON_FALLIBLE_API]);

impl<'tcx> LateLintPass<'tcx> for NonFallibleAPI {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'_>) {}
}
