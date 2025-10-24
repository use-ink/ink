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

#![doc(
    html_logo_url = "https://use.ink/img/crate-docs/logo.png",
    html_favicon_url = "https://use.ink/crate-docs/favicon.png"
)]
#![feature(rustc_private)]
#![feature(box_patterns)]

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_type_ir;

pub use parity_clippy_utils as clippy;

use if_chain::if_chain;
use rustc_hir::{
    ExprKind,
    HirId,
    ItemId,
    ItemKind,
    QPath,
    StmtKind,
    Ty,
    TyKind,
    def::DefKind,
    def_id::{
        DefId,
        LOCAL_CRATE,
    },
};
use rustc_lint::LateContext;
use rustc_middle::ty::{
    TyCtxt,
    fast_reject::SimplifiedType,
};
use rustc_span::Symbol;

/// Returns `DefId` of the `__ink_StorageMarker` marker trait (if any).
///
/// # Developer Note
///
/// In ink! 6.0.0, our code generation implements the marker trait `__ink_StorageMarker`
/// for the `#[ink(storage)]` annotated `struct`.
///
/// This marker trait can be used to find the ink! storage struct.
///
/// This approach is similar to Rust's use of [marker][rust-markers] traits
/// to express that a type satisfies some property.
///
/// [rust-markers]: https://doc.rust-lang.org/std/marker/index.html
fn storage_marker_trait(tcx: TyCtxt) -> Option<&DefId> {
    tcx.traits(LOCAL_CRATE).iter().find(|trait_def_id| {
        tcx.item_name(**trait_def_id).as_str() == "__ink_StorageMarker"
    })
}

/// Returns `true` iff the ink storage attribute is defined for the given HIR
///
/// # Developer Note
///
/// **IMPORTANT**: The description below doesn't apply to ink! 6.0.0,
/// because this mechanism is now replaced with the more idiomatic approach
/// of using a "marker" trait to identify the ink! storage `struct`
/// (see [`storage_marker_trait`] for details).
///
/// In ink! 5.0.0 our code generation added the annotation
/// `#[cfg(not(feature = "__ink_dylint_Storage"))] to contracts. This
/// allowed dylint to identify the storage struct in a contract.
///
/// Starting with Rust 1.81, `cargo` throws a warning for features that
/// are not declared in the `Cargo.toml` and also for not well-known
/// key-value pairs.
///
/// We don't want to burden contract developers with putting features that
/// are just for internal use there. The only alternative we found is to
/// use an obscure `cfg` condition, that is highly unlikely to be ever
/// annotated in a contract by a developer. Hence, we decided to use
/// `#[cfg(not(target_vendor = "fortanix"))]`, as it seems unlikely that a
/// contract will ever be compiled for this target.
///
/// We have to continue checking for the `__ink_dylint_Storage` attribute
/// here, as the linting will otherwise stop working for ink! 5.0.0 contracts.
fn has_storage_attr(cx: &LateContext, hir: HirId) -> bool {
    const INK_STORAGE_1: &str = "__ink_dylint_Storage";
    const INK_STORAGE_2: &str = "fortanix";
    let attrs = format!("{:?}", cx.tcx.hir_attrs(hir));
    attrs.contains(INK_STORAGE_1) || attrs.contains(INK_STORAGE_2)
}

/// Returns `DefId` of the `struct` annotated with `#[ink(storage)]` (if any).
///
/// See [`storage_marker_trait`].
pub fn find_storage_struct_def(tcx: TyCtxt) -> Option<&DefId> {
    let storage_marker_trait_id = storage_marker_trait(tcx)?;
    let storage_marker_impls = tcx
        .trait_impls_of(storage_marker_trait_id)
        .non_blanket_impls();
    debug_assert_eq!(
        storage_marker_impls.len(),
        1,
        "Expected exactly one implementation of the `__ink_StorageMarker` marker trait"
    );
    let (storage_ty, _) = storage_marker_impls.first()?;
    let SimplifiedType::Adt(adt_def_id) = storage_ty else {
        return None;
    };
    matches!(tcx.def_kind(adt_def_id), DefKind::Struct).then_some(adt_def_id)
}

/// Returns `ItemId` of the structure annotated with `#[ink(storage)]`
pub fn find_storage_struct(cx: &LateContext, item_ids: &[ItemId]) -> Option<ItemId> {
    let tcx = cx.tcx;
    // Finds storage item using `__ink_StorageMarker` marker trait for ink! 6.0.0
    // (see [`storage_marker_trait`] for details).
    let storage_item_id = find_storage_struct_def(tcx)
        .and_then(|def_id| def_id.as_local())
        .map(|local_def_id| tcx.local_def_id_to_hir_id(local_def_id))
        .map(|hir_id| {
            // See `Item::hir_id` for reverse operation.
            ItemId {
                owner_id: hir_id.owner,
            }
        });
    storage_item_id.or_else(|| {
        // Fallback for ink! 5.0.0 (see [`has_storage_attr`] for details).
        item_ids
            .iter()
            .find(|&item_id| {
                let item = cx.tcx.hir_item(*item_id);
                if_chain! {
                    if has_storage_attr(cx, item.hir_id());
                    if let ItemKind::Struct(..) = item.kind;
                    then { true } else { false }

                }
            })
            .copied()
    })
}

/// Returns `ItemId`s defined inside the code block of `const _: () = {}`.
///
/// The Rust code expanded after ink! code generation used these to define different
/// implementations of a contract.
fn items_in_unnamed_const(cx: &LateContext<'_>, id: &ItemId) -> Vec<ItemId> {
    if_chain! {
        if let ItemKind::Const(_, ty, _, body_id) = cx.tcx.hir_item(*id).kind;
        if let TyKind::Tup([]) = ty.kind;
        let body = cx.tcx.hir_body(body_id);
        if let ExprKind::Block(block, _) = body.value.kind;
        then {
            block.stmts.iter().fold(Vec::new(), |mut acc, stmt| {
                if let StmtKind::Item(id) = stmt.kind {
                    // We don't call `items_in_unnamed_const` here recursively, because the source
                    // code generated by ink! doesn't have nested `const _: () = {}` expressions.
                    acc.push(id)
                }
                acc
            })
        } else {
            vec![]
        }
    }
}

/// Collect all the `ItemId`s in nested `const _: () = {}`
pub fn expand_unnamed_consts(cx: &LateContext<'_>, item_ids: &[ItemId]) -> Vec<ItemId> {
    item_ids.iter().fold(Vec::new(), |mut acc, item_id| {
        acc.push(*item_id);
        acc.append(&mut items_in_unnamed_const(cx, item_id));
        acc
    })
}

/// Finds type of the struct that implements a contract with user-defined code
fn find_contract_ty_hir<'tcx>(
    cx: &LateContext<'tcx>,
    item_ids: &[ItemId],
) -> Option<&'tcx Ty<'tcx>> {
    item_ids
        .iter()
        .find_map(|item_id| {
            if_chain! {
                let item = cx.tcx.hir_item(*item_id);
                if let ItemKind::Impl(item_impl) = &item.kind;
                if let Some(trait_ref) = cx.tcx.impl_trait_ref(item.owner_id);
                if match_def_path(
                    cx,
                    trait_ref.skip_binder().def_id,
                    &["ink_primitives", "contract", "ContractEnv"],
                );
                then { Some(&item_impl.self_ty) } else { None }
            }
        })
        .copied()
}

/// Copied from <https://github.com/trailofbits/dylint/blob/3fcec25488436faef3700d09e56dbb588ba8c8a5/internal/src/match_def_path.rs#L31-L42>.
///
/// Checks if the given `DefId` matches any of the paths. Returns the index of matching
/// path, if any.
pub fn match_any_def_paths(
    cx: &LateContext<'_>,
    did: DefId,
    paths: &[&[&str]],
) -> Option<usize> {
    let search_path = cx.get_def_path(did);
    paths.iter().position(|p| {
        p.iter()
            .map(|x| Symbol::intern(x))
            .eq(search_path.iter().cloned())
    })
}

/// Copied from <https://github.com/trailofbits/dylint/blob/3fcec25488436faef3700d09e56dbb588ba8c8a5/internal/src/match_def_path.rs#L44-L51>.
///
/// Checks if the given `DefId` matches the path.
pub fn match_def_path(cx: &LateContext<'_>, did: DefId, syms: &[&str]) -> bool {
    // We should probably move to Symbols in Clippy as well rather than interning every
    // time.
    let path = cx.get_def_path(did);
    syms.iter()
        .map(|x| Symbol::intern(x))
        .eq(path.iter().copied())
}

/// Copied from <https://github.com/rust-lang/rust-clippy/blob/rust-1.86.0/clippy_utils/src/lib.rs#L507-L533>.
///
/// THIS METHOD IS DEPRECATED and will eventually be removed since it does not match
/// against the entire path or resolved `DefId`. Prefer using `match_def_path`. Consider
/// getting a `DefId` from `QPath::Resolved.1.res.opt_def_id()`.
///
/// Matches a `Path` against a slice of segment string literals.
///
/// There is also `match_qpath` if you are dealing with a `rustc_hir::QPath` instead of a
/// `rustc_hir::Path`.
///
/// # Examples
///
/// ```rust,ignore
/// if match_path(&trait_ref.path, &paths::HASH) {
///     // This is the `std::hash::Hash` trait.
/// }
///
/// if match_path(ty_path, &["rustc", "lint", "Lint"]) {
///     // This is a `rustc_middle::lint::Lint`.
/// }
/// ```
pub fn match_path(path: &rustc_hir::Path<'_>, segments: &[&str]) -> bool {
    path.segments
        .iter()
        .rev()
        .zip(segments.iter().rev())
        .all(|(a, b)| a.ident.name.as_str() == *b)
}

/// Compares types of two user-defined structs
fn eq_hir_struct_tys(lhs: &Ty<'_>, rhs: &Ty<'_>) -> bool {
    match (lhs.kind, rhs.kind) {
        (
            TyKind::Path(QPath::Resolved(_, lhs_path)),
            TyKind::Path(QPath::Resolved(_, rhs_path)),
        ) => lhs_path.res.eq(&rhs_path.res),
        _ => false,
    }
}

/// Finds an ID of the implementation of the contract struct containing user-defined code
pub fn find_contract_impl_id(
    cx: &LateContext<'_>,
    item_ids: Vec<ItemId>,
) -> Option<ItemId> {
    let contract_struct_ty = find_contract_ty_hir(cx, &item_ids)?;
    item_ids
        .iter()
        .find(|item_id| {
            if_chain! {
                let item = cx.tcx.hir_item(**item_id);
                if let ItemKind::Impl(item_impl) = &item.kind;
                if item_impl.of_trait.is_none();
                if eq_hir_struct_tys(contract_struct_ty, item_impl.self_ty);
                then { true } else { false }
            }
        })
        .copied()
}
