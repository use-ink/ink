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
    clippy::diagnostics::span_lint_hir_and_then,
    expand_unnamed_consts,
    find_contract_impl_id,
    match_any_def_paths,
    match_def_path,
};
use rustc_errors::Applicability;
use rustc_hir::{
    self as hir,
    def_id::DefId,
    AssocItemKind,
    ItemKind,
};
use rustc_index::bit_set::DenseBitSet;
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::{
    mir::{
        traversal,
        visit::Visitor,
        BasicBlock,
        BinOp,
        Body,
        BorrowKind,
        CallReturnPlaces,
        ConstOperand,
        HasLocalDecls,
        Local,
        Location,
        Operand,
        Place,
        Rvalue,
        Statement,
        Terminator,
        TerminatorEdges,
        TerminatorKind,
    },
    ty as mir_ty,
};
use rustc_mir_dataflow::Analysis;
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::{
    source_map::Spanned,
    Span,
};
use std::collections::{
    HashMap,
    HashSet,
};

declare_lint! {
    /// ## What it does
    /// Looks for strict equalities with balance in ink! contracts.
    ///
    /// ## Why is this bad?
    /// The problem with strict balance equality is that it is always possible to forcibly send
    /// tokens to a contract. For example, using
    /// [`terminate_contract`](https://use-ink.github.io/ink/ink_env/fn.terminate_contract.html).
    /// In such a case, the condition involving the contract balance will work incorrectly, what
    /// may lead to security issues, including DoS attacks and draining contract's gas.
    ///
    /// ## Known problems
    /// There are many ways to implement balance comparison in ink! contracts. This lint is not
    /// trying to be exhaustive. Instead, it addresses the most common cases that may occur in
    /// real-world contracts and focuses on precision and lack of false positives.
    ///
    /// ## Example
    /// Assume, there is an attacker contract that sends all its funds to the target contract when
    /// terminated:
    ///
    /// ```rust
    /// #[ink::contract]
    /// pub mod attacker {
    ///   // ...
    ///   #[ink(message)]
    ///   pub fn attack(&mut self, target: &AccountId) {
    ///       self.env().terminate_contract(target);
    ///   }
    /// }
    /// ```
    ///
    /// If the target contains a condition with strict balance equality, this may be manipulated by
    /// the attacker:
    ///
    /// ```rust
    /// #[ink::contract]
    /// pub mod target {
    ///   // ...
    ///   #[ink(message)]
    ///   pub fn do_something(&mut self) {
    ///       if self.env().balance() != 100 { // Bad: Strict balance equality
    ///           // ... some logic
    ///       }
    ///   }
    /// }
    /// ```
    ///
    /// This could be mitigated using non-strict equality operators in the condition with the
    /// balance:
    ///
    /// ```rust
    /// #[ink::contract]
    /// pub mod target {
    ///   // ...
    ///   #[ink(message)]
    ///   pub fn do_something(&mut self) {
    ///       if self.env().balance() < 100 { // Good: Non-strict equality
    ///           // ... some logic
    ///       }
    ///   }
    /// }
    /// ```
    pub STRICT_BALANCE_EQUALITY,
    Warn,
    "strict equality with contract's balance"
}

declare_lint_pass!(StrictBalanceEquality => [STRICT_BALANCE_EQUALITY]);

/// The dataflow problem that tracks local variables that are tainted with the return
/// value of `self.env().balance()`. The tainted values could be propagated between
/// function calls.
struct StrictBalanceEqualityAnalysis<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    fun_cache: &'a mut VisitedFunctionsCache,
    init_taints: TaintedArgs,
    mutable_references: MutableReferences,
}

/// Holds the results of running the dataflow analysis over a function with the given
/// input parameters.
type VisitedFunctionsCache = HashMap<(FunctionName, TaintedArgs), AnalysisResults>;
type FunctionName = String;
type TaintedArgs = Vec<bool>;
type AnalysisResults = DenseBitSet<Local>;

/// TransferFunction is a temporary object used by the implementation of a dataflow
/// transfer function to iterate over MIR statements of a function.
struct TransferFunction<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    fun_cache: &'a mut VisitedFunctionsCache,
    state: &'a mut DenseBitSet<Local>,
    mutable_references: &'a mut MutableReferences,
}

impl<'a, 'tcx> TransferFunction<'a, 'tcx> {
    pub fn new(
        cx: &'a LateContext<'tcx>,
        fun_cache: &'a mut VisitedFunctionsCache,
        state: &'a mut DenseBitSet<Local>,
        mutable_references: &'a mut MutableReferences,
    ) -> Self {
        Self {
            cx,
            fun_cache,
            state,
            mutable_references,
        }
    }
}

/// Reference Local |-> Origin Local
type MutableReferences = HashMap<Local, Local>;

impl<'a, 'tcx> StrictBalanceEqualityAnalysis<'a, 'tcx> {
    /// Should be called on contract functions that don't have input arguments tainted
    /// with balance
    pub fn new(
        cx: &'a LateContext<'tcx>,
        fun_cache: &'a mut VisitedFunctionsCache,
    ) -> Self {
        Self {
            cx,
            fun_cache,
            init_taints: TaintedArgs::new(),
            mutable_references: MutableReferences::new(),
        }
    }

    /// Should be called on private functions that may have input arguments tainted with
    /// balance
    pub fn new_with_arg_taints(
        cx: &'a LateContext<'tcx>,
        fun_cache: &'a mut VisitedFunctionsCache,
        init_taints: TaintedArgs,
    ) -> Self {
        Self {
            cx,
            fun_cache,
            init_taints,
            mutable_references: MutableReferences::new(),
        }
    }
}

/// The implementation of the transfer function for the dataflow problem
impl<'tcx> Analysis<'tcx> for StrictBalanceEqualityAnalysis<'_, 'tcx> {
    /// A lattice that represents program's state. `DenseBitSet` is a powerset over MIR
    /// Locals defined in the analyzed function. Inclusion to the set means that the
    /// Local is tainted with some operation with `self.env().balance()`.
    type Domain = DenseBitSet<Local>;

    const NAME: &'static str = "strict_balance_equality";

    fn bottom_value(&self, body: &Body) -> Self::Domain {
        // bottom = no balance taints
        DenseBitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, fn_mir: &Body, state: &mut Self::Domain) {
        // Initial source of taints are input arguments and contract fields
        if !self.init_taints.is_empty() {
            self.init_taints.iter().zip(fn_mir.args_iter()).for_each(
                |(init_taint, callee_local)| {
                    if *init_taint {
                        state.insert(callee_local);
                    }
                },
            )
        }
    }

    fn apply_primary_statement_effect(
        &mut self,
        state: &mut Self::Domain,
        statement: &Statement,
        location: Location,
    ) {
        TransferFunction::new(
            self.cx,
            self.fun_cache,
            state,
            &mut self.mutable_references,
        )
        .visit_statement(statement, location);
    }

    fn apply_primary_terminator_effect<'mir>(
        &mut self,
        state: &mut Self::Domain,
        terminator: &'mir Terminator<'tcx>,
        location: Location,
    ) -> TerminatorEdges<'mir, 'tcx> {
        TransferFunction::new(
            self.cx,
            self.fun_cache,
            state,
            &mut self.mutable_references,
        )
        .visit_terminator(terminator, location);
        terminator.edges()
    }

    fn apply_call_return_effect(
        &mut self,
        _state: &mut Self::Domain,
        _block: BasicBlock,
        _return_place: CallReturnPlaces,
    ) {
        // Do nothing
    }
}

impl Visitor<'_> for TransferFunction<'_, '_> {
    fn visit_assign(&mut self, place: &Place, rvalue: &Rvalue, _: Location) {
        match rvalue {
            // Direct comparison with the balance or propagation to a value tainted with
            // some operation with the balance
            Rvalue::BinaryOp(binop, box (lhs, rhs))
                if self.binop_strict_eq(binop) || self.binop_other(binop) =>
            {
                if tainted_with_balance(self.state, lhs).is_some()
                    || tainted_with_balance(self.state, rhs).is_some()
                {
                    self.state.insert(place.local);
                }
            }
            // Assignments of intermediate locals created by rustc
            Rvalue::Use(Operand::Move(use_place) | Operand::Copy(use_place)) => {
                let use_local = use_place.local;
                if self.state.contains(use_local) {
                    self.state.insert(place.local);
                }
            }
            // Values tainted with balance operation are propagated through references
            Rvalue::Ref(_, borrow_type, use_place) => {
                let use_local = use_place.local;
                if self.state.contains(use_local) {
                    self.state.insert(place.local);
                }
                if let BorrowKind::Mut { .. } = borrow_type {
                    self.mutable_references.insert(place.local, use_local);
                }
            }
            _ => {}
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator, _: Location) {
        if let TerminatorKind::Call {
            func,
            args,
            destination,
            ..
        } = &terminator.kind
        {
            if_chain! {
                if let Some((fn_def_id, _)) = func.const_fn_def();
                if match_def_path(self.cx,
                                  fn_def_id,
                                  &["ink", "env_access", "EnvAccess", "balance"]);
                then {
                    // Handle `self.env().balance()` calls
                    self.state.insert(destination.local);
                } else {
                    // Handle other calls
                    if let Operand::Constant(func_op) = func {
                        self.visit_call(func_op, args, destination)
                    }
                }
            };
        }
    }
}

impl TransferFunction<'_, '_> {
    fn binop_strict_eq(&self, binop: &BinOp) -> bool {
        matches!(binop, BinOp::Eq | BinOp::Ne)
    }
    fn binop_other(&self, binop: &BinOp) -> bool {
        matches!(
            binop,
            BinOp::Add
                | BinOp::Sub
                | BinOp::Mul
                | BinOp::Div
                | BinOp::Rem
                | BinOp::BitXor
                | BinOp::BitAnd
                | BinOp::BitOr
                | BinOp::Shl
                | BinOp::Shr
                | BinOp::Offset
        )
    }

    /// Returns all the origins of the given mutable reference.
    ///
    /// A mutable reference can have multiple origins because of compiler's two-phase
    /// borrows: https://rustc-dev-guide.rust-lang.org/borrow_check/two_phase_borrows.html
    fn get_mut_ref_origins(&self, ref_local: &Local) -> HashSet<Local> {
        let mut origins = HashSet::new();
        let _ = self.mutable_references.get(ref_local).map(|origin| {
            origins.insert(*origin);
            origins.extend(self.get_mut_ref_origins(origin));
        });
        origins
    }

    /// Returns true iff the return value of function is tainted with
    /// `self.env().balance()`
    fn is_return_value_tainted(&self, fn_state: &DenseBitSet<Local>) -> bool {
        let return_local = Place::return_place().local;
        fn_state.contains(return_local)
    }

    // Returns all the locals that correspond to mutable input arguments that were tainted
    // with balance after calling the function.
    fn get_tainted_input_args(
        &self,
        input_args: &[Spanned<Operand<'_>>],
        fn_mir: &Body,
        fn_state: &DenseBitSet<Local>,
    ) -> Vec<Local> {
        input_args.iter().zip(fn_mir.args_iter()).fold(
            Vec::new(),
            |mut acc, (caller_op, callee_local)| {
                if_chain! {
                    if fn_state.contains(callee_local);
                    if let Some(caller_place) = caller_op.node.place();
                    then {
                        let ref_local = caller_place.local;
                        acc.push(ref_local);
                        self.get_mut_ref_origins(&ref_local)
                            .iter()
                            .for_each(|origin| acc.push(*origin));
                    }
                };
                acc
            },
        )
    }

    fn fn_is_defined_in_user_code(&self, fn_def_id: &DefId) -> bool {
        fn_def_id.is_local()
    }

    fn visit_call(
        &mut self,
        func: &ConstOperand,
        args: &[Spanned<Operand<'_>>],
        destination: &Place,
    ) {
        let init_taints = args.iter().fold(Vec::new(), |mut acc, arg| {
            if let Operand::Move(place) | Operand::Copy(place) = arg.node {
                acc.push(self.state.contains(place.local))
            }
            acc
        });

        let fn_def_id =
            if let mir_ty::TyKind::FnDef(fn_def_id, _) = func.const_.ty().kind() {
                fn_def_id
            } else {
                return
            };

        // Handle `PartialEq` functions that implement comparison for non-primitive types,
        // including references like `&i32`.
        if_chain! {
            if init_taints.len() == 2;
            if init_taints.iter().any(|&tainted| tainted);
            if match_any_def_paths(
                self.cx,
                *fn_def_id,
                &[
                    &["core", "cmp", "PartialEq", "ne"],
                    &["core", "cmp", "PartialEq", "eq"],
                ],
            )
            .is_some();
            then {
                self.state.insert(destination.local);
                return
            }
        }

        let fn_mir = if_chain! {
            if self.fn_is_defined_in_user_code(fn_def_id);
            then { self.cx.tcx.optimized_mir(fn_def_id) } else { return }
        };

        // Run the dataflow analysis if the function hasn't been analyzed yet
        let cache_key = (func.to_string(), init_taints.clone());
        let analysis_results = if let Some(cached_results) =
            self.fun_cache.get(&cache_key)
        {
            cached_results
        } else {
            // Insert an empty value to handle recursive calls
            let _ = self
                .fun_cache
                .insert(cache_key.clone(), DenseBitSet::new_empty(0));
            let mut taint_results = StrictBalanceEqualityAnalysis::new_with_arg_taints(
                self.cx,
                self.fun_cache,
                init_taints,
            )
            .iterate_to_fixpoint(self.cx.tcx, fn_mir, None)
            .into_results_cursor(fn_mir);
            let taint_results =
                if let Some((last, _)) = traversal::reverse_postorder(fn_mir).last() {
                    // Reset to the dataflow state immediately after the terminator
                    taint_results.seek_to_block_end(last);
                    taint_results.get().clone()
                } else {
                    return
                };
            let _ = self.fun_cache.insert(cache_key.clone(), taint_results);
            if let Some(results) = self.fun_cache.get(&cache_key) {
                results
            } else {
                return
            }
        };

        // Recursive call of the function with the same input arguments
        if analysis_results.is_empty() {
            return
        }

        if self.is_return_value_tainted(analysis_results) {
            self.state.insert(destination.local);
        }

        self.get_tainted_input_args(args, fn_mir, analysis_results)
            .iter()
            .for_each(|tainted_input_arg| {
                self.state.insert(*tainted_input_arg);
            })
    }
}

/// Returns Local if the given Operand is tainted with the balance in the `state` lattice
fn tainted_with_balance(state: &DenseBitSet<Local>, op: &Operand) -> Option<Local> {
    if_chain! {
        if let Some(place) = op.place();
        if state.contains(place.local);
        then { Some(place.local) } else { None }
    }
}

impl<'tcx> LateLintPass<'tcx> for StrictBalanceEquality {
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
                let mut fun_cache = VisitedFunctionsCache::new();
                contract_impl.items.iter().for_each(|impl_item| {
                    if let AssocItemKind::Fn { .. } = impl_item.kind {
                        self.check_contract_fun(
                            cx,
                            &mut fun_cache,
                            impl_item.span,
                            impl_item.id.owner_id.to_def_id(),
                        )
                    }
                })
            }
        }
    }
}

impl StrictBalanceEquality {
    /// Checks a function from the contract implementation
    fn check_contract_fun(
        &mut self,
        cx: &LateContext<'_>,
        fun_cache: &mut VisitedFunctionsCache,
        fn_span: Span,
        fn_def_id: DefId,
    ) {
        let fn_mir = cx.tcx.optimized_mir(fn_def_id);
        let mut taint_results = StrictBalanceEqualityAnalysis::new(cx, fun_cache)
            .iterate_to_fixpoint(cx.tcx, fn_mir, None)
            .into_results_cursor(fn_mir);
        for (bb, bb_data) in traversal::preorder(fn_mir) {
            taint_results.seek_to_block_end(bb);
            let tainted_locals = taint_results.get();
            if tainted_locals.is_empty() {
                continue
            }
            let terminator = bb_data.terminator();
            if_chain! {
                if let TerminatorKind::SwitchInt { discr, .. } = &terminator.kind;
                if let Some(place) = discr.place();
                if tainted_locals.contains(place.local);
                let span = terminator.source_info.span;
                let scope = terminator.source_info.scope;
                let node = fn_mir.source_scopes[scope]
                    .local_data
                    .as_ref()
                    .unwrap_crate_local()
                    .lint_root;
                then {
                    let sugg_span = Span::new(
                            span.lo(),
                            span.hi(),
                            // We have to use a span different from `span`, since it is resulted
                            // after macro expansion and therefore cannot be used to emit
                            // diagnostics.
                            fn_span.ctxt(),
                            fn_span.parent()
                    );
                    span_lint_hir_and_then(
                        cx,
                        STRICT_BALANCE_EQUALITY,
                        node,
                        sugg_span,
                        "dangerous strict balance equality",
                        |diag| {
                            diag.span_suggestion(
                                sugg_span,
                                "consider using non-strict equality operators instead: `<`, `>`".to_string(),
                                "",
                                Applicability::Unspecified,
                            );
                            diag.help(
                                "for further information visit https://use.ink/linter/rules/strict_balance_equality".to_string(),
                            );
                        },
                    )
                }
            }
        }
    }
}
