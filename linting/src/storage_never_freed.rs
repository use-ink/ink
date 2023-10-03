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
    intravisit::{
        walk_body,
        walk_expr,
        Visitor,
    },
    Expr,
    ExprKind,
    ImplItemKind,
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
use std::collections::{
    btree_map::Entry,
    BTreeMap,
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
    // TODO: replace w/ ids
    pub has_insert: bool,
    pub has_remove: bool,
}
type FieldName = String;
type FieldsMap = BTreeMap<FieldName, FieldInfo>;

// https://paritytech.github.io/ink/ink_prelude/vec/struct.Vec.html
const VEC_INSERT_OPERATIONS: [&str; 6] = [
    "append",
    "extend_from_slice",
    "extend_from_within",
    "insert",
    "push",
    "push_with_capacity",
];
const VEC_REMOVE_OPERATIONS: [&str; 8] = [
    "clear",
    "dedup",
    "pop",
    "remove",
    "retain",
    "retain_mut",
    "swap_remove",
    "truncate",
];
const VEC_IGNORE_OPERATIONS: [&str; 2] = ["as_mut_ptr", "as_mut_slice"];

// https://paritytech.github.io/ink/ink_storage/struct.Mapping.html
const MAP_INSERT_OPERATIONS: [&str; 1] = ["insert"];
const MAP_REMOVE_OPERATIONS: [&str; 2] = ["remove", "take"];

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
fn find_collection_fields(cx: &LateContext, storage_struct_id: ItemId) -> FieldsMap {
    let mut result = FieldsMap::new();
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
                    let field_name = field_def.ident.name.as_str();
                    // TODO: Inspect type aliases
                    if let Some(_did) = find_vec_did(cx, path) {
                        result.insert(field_name.to_string(), FieldInfo::new(field_def.def_id, CollectionTy::Vec));
                        return;
                    }
                    if let Some(_did) = find_map_did(cx, path) {
                        result.insert(field_name.to_string(), FieldInfo::new(field_def.def_id, CollectionTy::Map));
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

/// Visitor that collects `insert` and `remove` operations
struct InsertRemoveCollector<'a, 'b, 'tcx> {
    cx: &'tcx LateContext<'a>,
    fields: &'b mut FieldsMap,
}

impl<'a, 'b, 'tcx> InsertRemoveCollector<'a, 'b, 'tcx> {
    fn new(cx: &'tcx LateContext<'a>, fields: &'b mut FieldsMap) -> Self {
        Self { cx, fields }
    }

    /// Finds a field of the supported type in the given expression present with the form
    /// `self.field_name`
    fn find_field_name(&self, e: &Expr) -> Option<String> {
        if_chain! {
            if let ExprKind::Field(s, field) = &e.kind;
            if let ExprKind::Path(ref path) = s.kind;
            let ty = self.cx.qpath_res(path, s.hir_id);
            // TODO: check if ty is `self`
            then { Some(field.name.as_str().to_string()) } else { None }
        }
    }
}

impl<'hir> Visitor<'hir> for InsertRemoveCollector<'_, '_, '_> {
    fn visit_expr(&mut self, e: &'hir Expr<'hir>) {
        match &e.kind {
            ExprKind::Assign(lhs, ..) => {
                if_chain! {
                    if let ExprKind::Index(field, _) = lhs.kind;
                    if let Some(field_name) = self.find_field_name(field);
                    then {
                        self.fields
                            .entry(field_name.to_string())
                            .and_modify(|field_info| {
                                field_info.has_insert = true;
                            });
                    }
                }
            }
            ExprKind::MethodCall(method_path, receiver, args, _) => {
                args.iter().for_each(|arg| walk_expr(self, arg));
                if_chain! {
                    if let Some(field_name) = self.find_field_name(receiver);
                    if let Entry::Occupied(mut e) = self.fields.entry(field_name.to_string());
                    let method_name = method_path.ident.as_str();
                    then {
                        let field_info = e.get_mut();
                        match field_info.ty {
                            CollectionTy::Vec => {
                                if VEC_IGNORE_OPERATIONS.contains(&method_name) {
                                    e.remove();
                                } else if VEC_INSERT_OPERATIONS.contains(&method_name) {
                                    field_info.has_insert = true;
                                } else if VEC_REMOVE_OPERATIONS.contains(&method_name) {
                                    field_info.has_remove = true;
                                }
                            },
                            CollectionTy::Map => {
                                if MAP_INSERT_OPERATIONS.contains(&method_name) {
                                    field_info.has_insert = true;
                                } else if MAP_REMOVE_OPERATIONS.contains(&method_name) {
                                    field_info.has_remove = true;
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        walk_expr(self, e);
    }
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
                    let impl_item = cx.tcx.hir().impl_item(impl_item.id);
                    if let ImplItemKind::Fn(_, fn_body_id) = impl_item.kind {
                        let mut visitor = InsertRemoveCollector::new(cx, &mut fields);
                        walk_body(&mut visitor, cx.tcx.hir().body(fn_body_id));
                    }
                });
                fields.iter().for_each(|(_, field)| {
                    if field.has_insert && !field.has_remove {
                        report_field(cx, field)
                    }
                })
            }
        }
    }
}
