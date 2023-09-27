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
use crate::ink_utils::{
    expand_unnamed_consts,
    find_contract_impl_id,
    find_storage_struct,
};
use clippy_utils::{
    diagnostics::span_lint_and_then,
    is_lint_allowed,
    match_def_path,
    match_path,
    source::snippet_opt,
};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::{
    self as hir,
    def::{
        DefKind,
        Res,
    },
    def_id::{
        DefId,
        LocalDefId,
    },
    AssocItemKind,
    ItemId,
    ItemKind,
    Node,
    Path,
    QPath,
    TyKind,
};
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

enum CollectionTy {
    Vec,
    Map,
}

/// Fields with collection types that should have both insert and remove operations
struct FieldInfo {
    pub did: LocalDefId,
    pub ty: CollectionTy,
    pub has_insert: bool,
    pub has_remove: bool,
}

impl FieldInfo {
    pub fn new(did: LocalDefId, ty: CollectionTy) -> Self {
        Self {
            did,
            ty,
            has_insert: false,
            has_remove: false,
        }
    }
}

/// Returns `DefId` of a field if it has the `Vec` type
fn find_vec_did(cx: &LateContext, path: &Path) -> Option<DefId> {
    if_chain! {
        if let Res::Def(DefKind::Struct, def_id) = path.res;
        if match_def_path(cx, def_id, &["alloc", "vec", "Vec"]);
        then { Some(def_id) } else { None }
    }
}

/// Returns `DefId` of a field if it has the `Mapping` type
fn find_map_did(cx: &LateContext, path: &Path) -> Option<DefId> {
    if_chain! {
        if let Res::Def(DefKind::Struct, def_id) = path.res;
        if match_def_path(cx, def_id, &["ink_storage", "lazy", "mapping", "Mapping"]);
        then { Some(def_id) } else { None }
    }
}

/// Returns vectors of fields that have collection types
fn find_collection_fields(cx: &LateContext, storage_struct_id: ItemId) -> Vec<FieldInfo> {
    let mut result = Vec::new();
    let item = cx.tcx.hir().item(storage_struct_id);
    if let ItemKind::Struct(var_data, _) = item.kind {
        var_data.fields().iter().for_each(|field_def| {
            if_chain! {
                // Collection fields of the storage are expanded like this:
                // vec_field: <Vec<
                //     AccountId,
                // > as ::ink::storage::traits::AutoStorableHint<
                //     ::ink::storage::traits::ManualKey<993959520u32, ()>,
                // >>::Type,
                if let TyKind::Path(QPath::Resolved(Some(ty), path)) = field_def.ty.kind;
                if match_path(path, &["ink", "storage", "traits", "AutoStorableHint", "Type"]);
                if let TyKind::Path(QPath::Resolved(None, path)) = ty.kind;
                then {
                    // TODO: Inspect type aliases
                    if let Some(_did) = find_vec_did(cx, path) {
                        result.push(FieldInfo::new(field_def.def_id, CollectionTy::Vec));
                        return;
                    }
                    if let Some(_did) = find_map_did(cx, path) {
                        result.push(FieldInfo::new(field_def.def_id, CollectionTy::Map));
                    }
                }
            }
        })
    };
    result
}

/// Reports the given field defintion
fn report_field(cx: &LateContext, field_info: &FieldInfo) {
    if_chain! {
        if let Node::Field(field) = cx.tcx.hir().get_by_def_id(field_info.did);
        if !is_lint_allowed(cx, STORAGE_NEVER_FREED, field.hir_id);
        then {
            span_lint_and_then(
                cx,
                STORAGE_NEVER_FREED,
                field.span,
                "storage never freed",
                |diag| {
                    let snippet = snippet_opt(cx, field.span).expect("snippet must exist");
                    diag.span_suggestion(
                        field.span,
                        "consider adding operations to remove elements available to the user".to_string(),
                        snippet,
                        Applicability::Unspecified,
                    );
                },
            )

        }
    }
}

/// Collects information about `insert` and `remove` operations in the body of the
/// function
fn collect_insert_remove_ops(fields: &mut Vec<FieldInfo>) {
    todo!()
}

impl<'tcx> LateLintPass<'tcx> for StorageNeverFreed {
    fn check_mod(
        &mut self,
        cx: &LateContext<'tcx>,
        m: &'tcx hir::Mod<'tcx>,
        _: hir::HirId,
    ) {
        if_chain! {
            // Find fields of Vec/Mapping type
            if let Some(storage_struct_id) = find_storage_struct(cx, m.item_ids);
            let mut fields = find_collection_fields(cx, storage_struct_id);
            if !fields.is_empty();
            // Find all the user-defined functions of the contract
            let all_item_ids = expand_unnamed_consts(cx, m.item_ids);
            if let Some(contract_impl_id) = find_contract_impl_id(cx, all_item_ids);
            let contract_impl = cx.tcx.hir().item(contract_impl_id);
            if let ItemKind::Impl(contract_impl) = contract_impl.kind;
            then {
                contract_impl.items.iter().for_each(|impl_item| {
                    if let AssocItemKind::Fn { .. } = impl_item.kind {
                        collect_insert_remove_ops(&mut fields);
                    }
                });
                fields.iter().for_each(|field| {
                    if field.has_insert && !field.has_remove {
                        report_field(cx, field)
                    }
                })
            }
        }
    }
}
