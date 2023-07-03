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
use clippy_utils::match_def_path;
use if_chain::if_chain;
use rustc_hir::{
    AssocItemKind,
    Expr,
    ExprKind,
    ImplItemKind,
    ImplItemRef,
    Item,
    ItemKind,
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
    /// **Known problems:** Events 2.0 currently are not supported:
    /// https://github.com/paritytech/ink/pull/1827.
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

/// Returns `true` if `item` is an implementation of `::ink::env::Topics` for a storage struct.
/// If that's the case, it returns the name of this struct.
fn is_ink_topics_impl<'tcx>(cx: &LateContext<'tcx>, item: &'tcx Item<'_>) -> bool {
    if let Some(trait_ref) = cx.tcx.impl_trait_ref(item.owner_id) {
        match_def_path(cx, trait_ref.0.def_id, &["ink_env", "topics", "Topics"])
    } else {
        false
    }
}

/// Returns `true` if `impl_item` is the `topics` function
fn is_topics_function(impl_item: &ImplItemRef) -> bool {
    impl_item.kind == AssocItemKind::Fn { has_self: true }
        && impl_item.ident.name.as_str() == "topics"
}

/// Returns `true` if `ty` is a primitive type that should not be annotated with
/// `#[ink(topic)]`
fn is_primitive_ty(ty: &Ty) -> bool {
    // dbg!(ty.kind());
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

/// Returns `true` if the type of the argument of `push_topic` has a primitive type
fn is_primitive_topic_ty(arg_ty: &Ty) -> bool {
    if_chain! {
    if let TyKind::Ref(_, prefixed_value_ty, _) = arg_ty.kind();
    if let ty::Adt(_, substs) = prefixed_value_ty.kind();
    if substs.len() == 3;
    if let ty::GenericArgKind::Type(ty) = substs[2].unpack();
    then { is_primitive_ty(&ty) }
    else { false }
    }
}

/// Checks the sequence of `push_topic` method calls raising warnings if the code was
/// generated from struct fields with primitive types.
fn check_push_topic_calls(cx: &LateContext, method_call: &Expr) {
    if_chain! {
    if let ExprKind::MethodCall(seg, receiver, [arg], _) = method_call.kind;
    if seg.ident.name.as_str() == "push_topic";
    then
    {
        if_chain! {
            if cx.tcx.has_typeck_results(arg.hir_id.owner.to_def_id());
            let ty = cx.tcx.typeck(arg.hir_id.owner.def_id).expr_ty(arg);
            then {
                // dbg!(is_primitive_topic_ty(&ty));
                is_primitive_topic_ty(&ty);
            }
        }
        check_push_topic_calls(cx, receiver)
    }
    }
}

impl<'tcx> LateLintPass<'tcx> for PrimitiveTopic {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'_>) {
        if_chain! {
            if !is_ink_topics_impl(cx, item);
            if let ItemKind::Impl(topics_impl) = &item.kind;
            then {
            topics_impl.items.iter().for_each(|impl_item| {
                if_chain! {
                // The example of the generated code we are interested in:
                // ```rust
                // impl ::ink::env::Topics for Transaction {
                //     // ...
                //     fn topics<E, B>(&self, /* ... */)
                //     {
                //         builder
                //             .build::<Self>()
                //             .push_topic /* ... */
                //             .push_topic::<
                //                 ::ink::env::topics::PrefixedValue<Option<AccountId>>,
                //             >(
                //                 &::ink::env::topics::PrefixedValue {
                //                     value: &self.src,
                //                     prefix: b"PrimitiveTopic::Transaction::src",
                //                 },
                //             )
                //             .push_topic /* ... */
                //             .finish(/*...*/);
                // ```
                if is_topics_function(&impl_item);
                let impl_item = cx.tcx.hir().impl_item(impl_item.id);
                if let ImplItemKind::Fn(_, eid) = impl_item.kind;
                let body = cx.tcx.hir().body(eid).value;
                if let ExprKind::Block (block, _) = body.kind;
                if let Some(build_call) = block.expr;
                if let ExprKind::MethodCall (_, finish_expr, ..) = build_call.kind;
                then {
                    check_push_topic_calls(cx, finish_expr);
                }
                }
            })
        }
        }
    }
}
