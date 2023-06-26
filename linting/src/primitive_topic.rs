// Copyright 2018-2023 Parity Technologies (UK) Ltd.
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
use rustc_hir::Item;
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};

declare_lint! {
    /// **What it does:** Checks for ink! contracts that use
    /// the [`#[ink(topic)]`](https://use.ink/macros-attributes/topic) annotation with primitive
    /// number types. Topics are discrete events for which it makes sense to filter. Typical
    /// examples of fields that should be filtered are `AccountId`, `bool` or `enum` variants.
    ///
    /// **Why is this bad?** It typically doesn't make sense to annotate types like `u32` or `i32`
    /// as a topic, if those fields can take continuous values that could be anywhere between
    /// `::MIN` and `::MAX`. An example of a case where it doesn't make sense at all to have a
    /// topic on the storage field is something like `value: Balance` in the examle below.
    ///
    /// **Known problems:**
    ///
    /// **Example:**
    ///
    /// ```rust
    /// // Good
    /// // Filtering transactions based on source and destination addresses.
    /// #[ink(event)]
    /// pub struct Transaction {
    ///     #[ink(topic)]
    ///     src: Option<AccountId>,
    ///     #[ink(topic)]
    ///     dst: Option<AccountId>,
    ///     value: Balance,
    /// }
    /// ```
    ///
    /// ```rust
    /// // Bad
    /// // It typically makes no sense to filter `Balance`, since its value may varies from `::MAX`
    /// // to `::MIN`.
    /// #[ink(event)]
    /// pub struct Transaction {
    ///     #[ink(topic)]
    ///     src: Option<AccountId>,
    ///     #[ink(topic)]
    ///     dst: Option<AccountId>,
    ///     #[ink(topic)]
    ///     value: Balance,
    /// }
    /// ```
    pub PRIMITIVE_TOPIC,
    Warn,
    "The `#[ink(topic)]` annotation should not be used with a number primitive"
}

declare_lint_pass!(PrimitiveTopic => [PRIMITIVE_TOPIC]);

impl<'tcx> LateLintPass<'tcx> for PrimitiveTopic {
    fn check_item(&mut self, _cx: &LateContext<'tcx>, _item: &'tcx Item<'_>) {
        todo!()
    }
}
