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
use rustc_hir as hir;
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};

declare_lint! {
    /// **What it does:**
    /// This lint ensures that for every storage field with a collection type that supports adding
    /// new elements, there's also an operation for removing elements.
    ///
    /// **Why is this bad?**
    /// When a user executes a contract function that writes to storage, the user has to put a
    /// deposit down for the amount of storage space used. Whoever frees up that storage at some
    /// later point gets the deposit back. Therefore, it is always a good idea to make it possible
    /// for users to free up their storage space.
    ///
    /// **Example:**
    ///
    /// In the following example there is a storage field with the `Mapping` type that has an
    /// function that inserts new elements:
    /// ```rust
    /// #[ink(storage)]
    /// pub struct Transaction {
    ///     values: Mapping<AccountId, AccountId>,
    /// }
    ///
    /// fn add_value(&mut self, k: &AccountId, v: &AccountId) {
    ///     // ...
    ///     self.values.insert(k, v);
    ///     // ...
    /// }
    /// ```
    ///
    /// But, ideally, there also should be a function that allows the user to remove elements from
    /// the Mapping freeing storage space:
    ///
    /// ```rust
    /// fn del_value(&mut self, k: &AccountId) {
    ///     // ...
    ///     self.values.remove(k);
    ///     // ...
    /// }
    /// ```
    pub STORAGE_NEVER_FREED,
    Allow,
    "storage never freed"
}

declare_lint_pass!(StorageNeverFreed => [STORAGE_NEVER_FREED]);

impl<'tcx> LateLintPass<'tcx> for StorageNeverFreed {
    fn check_mod(
        &mut self,
        cx: &LateContext<'tcx>,
        m: &'tcx hir::Mod<'tcx>,
        _: hir::HirId,
    ) {
    }
}
