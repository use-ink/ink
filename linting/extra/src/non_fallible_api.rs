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

use if_chain::if_chain;
use ink_linting_utils::{
    clippy::{
        diagnostics::span_lint_and_then,
        is_lint_allowed,
    },
    expand_unnamed_consts,
    find_contract_impl_id,
    match_def_path,
};
use rustc_errors::Applicability;
use rustc_hir::{
    self as hir,
    def_id::DefId,
    intravisit::{
        walk_body,
        walk_expr,
        Visitor,
    },
    Body,
    Expr,
    ExprKind,
    ImplItemKind,
    ItemKind,
    PathSegment,
};
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::{
    hir::nested_filter,
    ty::{
        self,
        ConstKind,
        Ty,
        TypeckResults,
        Value,
    },
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_type_ir::TyKind;

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
    /// For more context, see: [ink#1910](https://github.com/use-ink/ink/pull/1910).
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
    /// // Bad: can panic if `input_string` doesn't fit into the static buffer
    /// self.map.insert(input_string, &self.sender);
    /// ```
    ///
    /// It could be replaced with the fallible version of `Mapping::insert`:
    ///
    /// ```rust
    /// // Good: returns Result::Err on incorrect input
    /// self.map.try_insert(input_string, &self.sender);
    /// ```
    ///
    /// Otherwise, the user could explicitly check the encoded size of the argument in their code:
    ///
    /// ```rust
    /// // Good: explicitly checked encoded size of the input
    /// if String::encoded_size(&input_string) < ink_env::BUFFER_SIZE {
    ///   self.map.insert(input_string, &self.sender);
    /// }
    /// ```
    pub NON_FALLIBLE_API,
    Warn,
    "using non-fallible API"
}

declare_lint_pass!(NonFallibleAPI => [NON_FALLIBLE_API]);

#[derive(Debug)]
enum TyToCheck {
    Mapping,
    Lazy,
    StorageVec,
}

impl TyToCheck {
    pub fn try_from_adt(cx: &LateContext<'_>, did: DefId) -> Option<Self> {
        if match_def_path(cx, did, &["ink_storage", "lazy", "Lazy"]) {
            return Some(Self::Lazy)
        }

        if match_def_path(cx, did, &["ink_storage", "lazy", "mapping", "Mapping"]) {
            return Some(Self::Mapping)
        }
        if match_def_path(cx, did, &["ink_storage", "lazy", "vec", "StorageVec"]) {
            return Some(Self::StorageVec)
        }
        None
    }

    pub fn find_fallible_alternative(&self, method_name: &str) -> Option<String> {
        use TyToCheck::*;
        match self {
            Mapping => {
                match method_name {
                    "insert" => Some("try_insert".to_string()),
                    "get" => Some("try_get".to_string()),
                    "take" => Some("try_take".to_string()),
                    _ => None,
                }
            }
            Lazy => {
                match method_name {
                    "get" => Some("try_get".to_string()),
                    "set" => Some("try_set".to_string()),
                    _ => None,
                }
            }
            StorageVec => {
                match method_name {
                    "peek" => Some("try_peek".to_string()),
                    "get" => Some("try_get".to_string()),
                    "set" => Some("try_set".to_string()),
                    "pop" => Some("try_pop".to_string()),
                    "push" => Some("try_push".to_string()),
                    _ => None,
                }
            }
        }
    }
}

/// Visitor that finds usage of non-fallible calls in the bodies of functions
struct APIUsageChecker<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    maybe_typeck_results: Option<&'tcx TypeckResults<'tcx>>,
}

impl<'a, 'tcx> APIUsageChecker<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'tcx>) -> Self {
        Self {
            cx,
            maybe_typeck_results: cx.maybe_typeck_results(),
        }
    }

    /// Returns true iff the given type has statically known size when encoded with
    /// `scale_codec`
    fn is_statically_known(&self, ty: &Ty<'tcx>) -> bool {
        match ty.kind() {
            ty::Bool | ty::Char | ty::Int(_) | ty::Uint(_) | ty::Float(_) | ty::Str => {
                true
            }
            ty::Tuple(inner_tys) => {
                inner_tys.iter().all(|ty| self.is_statically_known(&ty))
            }
            ty::Ref(_, inner, _) => self.is_statically_known(inner),
            ty::Adt(adt_def, substs) => {
                adt_def.variants().iter().all(|variant| {
                    variant.fields.iter().all(|field| {
                        self.is_statically_known(&field.ty(self.cx.tcx, substs))
                    })
                })
            }
            ty::Array(inner_ty, len_const) => {
                if_chain! {
                    if self.is_statically_known(inner_ty);
                    if let ConstKind::Value(value) = len_const.kind();
                    if let Value { ty: _, valtree } = value;
                    let elements_size = valtree.unwrap_leaf().to_target_usize(self.cx.tcx);
                    if elements_size < (ink_env::BUFFER_SIZE as u64);
                    then { true } else { false }
                }
            }
            _ => false,
        }
    }

    /// Raises warnings if the given method call is potentially unsafe and could be
    /// replaced
    fn check_method_call(
        &self,
        receiver_ty: &TyToCheck,
        method_path: &PathSegment,
        method_name: &str,
        arg_ty: Ty<'tcx>,
    ) {
        if_chain! {
            if !self.is_statically_known(&arg_ty);
            if let Some(fallible_method) = receiver_ty.find_fallible_alternative(method_name);
            then {
                span_lint_and_then(
                    self.cx,
                    NON_FALLIBLE_API,
                    method_path.ident.span,
                    format!(
                        "using a non-fallible `{receiver_ty:?}::{method_name}` with an argument that may not fit into the static buffer",
                    ).as_str().to_owned(),
                    |diag| {
                        diag.span_suggestion(
                            method_path.ident.span,
                            format!("consider using `{fallible_method}`"),
                            "",
                            Applicability::Unspecified,
                        );
                        diag.help(
                            "for further information visit https://use.ink/linter/rules/non_fallible_api".to_string(),
                        );
                },
                )
            }
        }
    }
}

impl<'tcx> Visitor<'tcx> for APIUsageChecker<'_, 'tcx> {
    type NestedFilter = nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.cx.tcx
    }

    fn visit_expr(&mut self, e: &'tcx Expr<'tcx>) {
        if_chain! {
            if !is_lint_allowed(self.cx, NON_FALLIBLE_API, e.hir_id);
            if let ExprKind::MethodCall(method_path, receiver, _, _) = &e.kind;
            if let Some(typeck_results) = self.maybe_typeck_results;
            let ty = typeck_results.expr_ty(receiver);
            if let TyKind::Adt(def, substs) = ty.kind();
            if let Some(ty) = TyToCheck::try_from_adt(self.cx, def.0.0.did);
            then {
                substs
                    .iter()
                    .take(substs.len() - 1)
                    .filter_map(|subst| subst.as_type())
                    .for_each(|arg_ty| {
                        self.check_method_call(
                            &ty,
                            method_path,
                            &method_path.ident.to_string(),
                            arg_ty)
                    })
            }
        }
        walk_expr(self, e);
    }

    fn visit_body(&mut self, body: &Body<'tcx>) {
        let old_maybe_typeck_results = self
            .maybe_typeck_results
            .replace(self.cx.tcx.typeck_body(body.id()));
        walk_body(self, body);
        self.maybe_typeck_results = old_maybe_typeck_results;
    }
}

impl<'tcx> LateLintPass<'tcx> for NonFallibleAPI {
    fn check_mod(
        &mut self,
        cx: &LateContext<'tcx>,
        m: &'tcx hir::Mod<'tcx>,
        _: hir::HirId,
    ) {
        if_chain! {
            let all_item_ids = expand_unnamed_consts(cx, m.item_ids);
            if let Some(contract_impl_id) = find_contract_impl_id(cx, all_item_ids);
            let contract_impl = cx.tcx.hir_item(contract_impl_id);
            if let ItemKind::Impl(contract_impl) = contract_impl.kind;
            then {
                contract_impl.items.iter().for_each(|impl_item| {
                    let impl_item = cx.tcx.hir_impl_item(impl_item.id);
                    if let ImplItemKind::Fn(..) = impl_item.kind {
                        let mut visitor = APIUsageChecker::new(cx);
                        visitor.visit_impl_item(impl_item);
                    }
                })
            }
        }
    }
}
