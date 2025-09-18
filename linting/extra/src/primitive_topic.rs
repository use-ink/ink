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
        source::snippet_opt,
    },
    match_def_path,
};
use rustc_errors::Applicability;
use rustc_hash::FxHashSet;
use rustc_hir::{
    self,
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
    def::{
        DefKind,
        Res,
    },
    def_id::DefId,
};
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::ty::{
    self,
    FieldDef,
    Ty,
    TyCtxt,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::Symbol;

declare_lint! {
    /// ## What it does
    /// Checks for ink! contracts that use the
    /// [`#[ink(topic)]`](https://use.ink/macros-attributes/topic) annotation with primitive number
    /// types. Topics are discrete events for which it makes sense to filter. Typical examples of
    /// fields that should be filtered are `AccountId`, `bool` or `enum` variants.
    ///
    /// ## Why is this bad?
    /// It typically doesn't make sense to annotate types like `u32` or `i32` as a topic, if those
    /// fields can take continuous values that could be anywhere between `::MIN` and `::MAX`. An
    /// example of a case where it doesn't make sense at all to have a topic on the storage field
    /// is something like `value: Balance` in the example below.
    ///
    /// ## Example
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
    ///
    /// Use instead:
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
    pub PRIMITIVE_TOPIC,
    Warn,
    "`#[ink(topic)]` should not be used with a number primitive"
}

declare_lint_pass!(PrimitiveTopic => [PRIMITIVE_TOPIC]);

/// Returns `true` if `item` is an implementation of `::ink::env::Event` for a storage
/// struct.
fn is_ink_event_impl<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'_>) -> bool {
    if let Some(trait_ref) = cx.tcx.impl_trait_ref(item.owner_id) {
        match_def_path(
            cx,
            trait_ref.skip_binder().def_id,
            &["ink_env", "event", "Event"],
        )
    } else {
        false
    }
}

/// Returns `true` if `impl_item` is the `topics` function
fn is_topics_function(impl_item: &ImplItemRef) -> bool {
    impl_item.kind == AssocItemKind::Fn { has_self: true }
        && impl_item.ident.name.as_str() == "topics"
}

/// Returns `true` if `ty` is a numerical primitive type that should not be annotated with
/// `#[ink(topic)]`
fn is_primitive_number_ty(ty: &Ty) -> bool {
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

/// Reports a topic-annotated field with a numerical primitive type
fn report_field(cx: &LateContext, event_def_id: DefId, field_name: &str) {
    if_chain! {
        if let Some(Node::Item(event_node)) = cx.tcx.hir_get_if_local(event_def_id);
        if let ItemKind::Struct(_, ref struct_def, _) = event_node.kind;
        if let Some(field) = struct_def.fields().iter().find(|f|{ f.ident.as_str() == field_name });
        if !is_lint_allowed(cx, PRIMITIVE_TOPIC, field.hir_id);
        then {
            span_lint_and_then(
                cx,
                PRIMITIVE_TOPIC,
                field.span,
                "using `#[ink(topic)]` for a field with a primitive number type",
                |diag| {
                    let snippet = snippet_opt(cx, field.span).expect("snippet must exist");
                    diag.span_suggestion(
                        field.span,
                        "consider removing `#[ink(topic)]`".to_string(),
                        snippet,
                        Applicability::Unspecified,
                    );
                    diag.help(
                        "for further information visit https://use.ink/linter/rules/primitive_topic".to_string(),
                    );
                },
            )
        }
    }
}

/// Returns `DefId` of the event struct for which `Topics` is implemented
fn get_event_def_id(topics_impl: &Impl) -> Option<DefId> {
    if_chain! {
        if let rustc_hir::TyKind::Path(qpath) = &topics_impl.self_ty.kind;
        if let QPath::Resolved(_, path) = qpath;
        if let Res::Def(DefKind::Struct, def_id) = path.res;
        then { Some(def_id) }
        else { None }
    }
}

/// Returns true if the given field is annotated with an `#[ink(topic)]` attribute.
fn is_topic_field(field: &FieldDef, tcx: TyCtxt) -> bool {
    tcx.get_attrs(field.did, Symbol::intern("ink")).any(|attr| {
        let Some(meta_list) = attr.meta_item_list() else {
            return false;
        };
        meta_list.iter().any(|meta| {
            meta.has_name(Symbol::intern("topic"))
                || meta.lit().is_some_and(|lit| lit.symbol.as_str() == "topic")
        })
    })
}

impl<'tcx> LateLintPass<'tcx> for PrimitiveTopic {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'_>) {
        if_chain! {
            if let ItemKind::Impl(topics_impl) = &item.kind;
            if is_ink_event_impl(cx, item);
            if let Some(event_def_id) = get_event_def_id(topics_impl);
            then {
                // Collect names of topic fields.
                let event_fields = cx.tcx.adt_def(event_def_id).all_fields();
                let topic_fields: FxHashSet<_> = event_fields
                    .filter_map(|field| is_topic_field(field, cx.tcx).then_some(field.name))
                    .collect();
                topics_impl.items.iter().for_each(|impl_item| {
                    if_chain! {
                        // We need to extract field patterns from the event struct matched in the
                        // `topics` function to access their inferred types.
                        // Here is the simplified example of the expanded code:
                        // ```
                        // fn topics(/* ... */) {
                        //      match self {
                        //          MyEvent {
                        //              field_1: __binding_0,
                        //              field_2: __binding_1,
                        //              /* ... */
                        //              ..
                        //          } => { /* ... */ }
                        //     }
                        // }
                        // ```
                        if is_topics_function(impl_item);
                        let impl_item = cx.tcx.hir_impl_item(impl_item.id);
                        if let ImplItemKind::Fn(_, eid) = impl_item.kind;
                        let body = cx.tcx.hir_body(eid).value;
                        if let ExprKind::Block (block, _) = body.kind;
                        if let Some(match_self) = block.expr;
                        if let ExprKind::Match(_, [Arm { pat: arm_pat, .. }], _) = match_self.kind;
                        if let PatKind::Struct(_, pat_fields, _) = &arm_pat.kind;
                        then {
                            pat_fields
                                .iter()
                                .filter(|pat_field| topic_fields.contains(&pat_field.ident.name))
                                .for_each(|pat_field| {
                                    cx.tcx.has_typeck_results(pat_field.hir_id.owner.def_id)
                                        .then(|| {
                                            let topic_ty = cx.tcx
                                                .typeck(pat_field.hir_id.owner.def_id)
                                                .pat_ty(pat_field.pat).peel_refs();
                                            if is_primitive_number_ty(&topic_ty) {
                                                report_field(cx, event_def_id, pat_field.ident.as_str())
                                            }
                                        });
                                })
                        }
                    }
                })
            }
        }
    }
}
