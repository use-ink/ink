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
    match_def_path,
    source::snippet_opt,
};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::{
    self as hir,
    def_id::LocalDefId,
    intravisit::FnKind,
    FnDecl,
};
use rustc_index::bit_set::BitSet;
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::mir::{
    traversal,
    visit::Visitor,
    BasicBlock,
    BinOp,
    Body,
    HasLocalDecls,
    Local,
    Location,
    Operand,
    Place,
    Rvalue,
    Statement,
    Terminator,
    TerminatorKind,
};
use rustc_mir_dataflow::{
    Analysis,
    AnalysisDomain,
    CallReturnPlaces,
    Forward,
};
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::Span;

declare_lint! {
    /// **What it does:** Looks for strict equalities with balance in ink! contracts.
    ///
    /// **Why is this bad?** The problem with strict balance equality is that it is always possible
    /// to forcibly send tokens to a contract, for example, using
    /// [`terminate_contract`](https://paritytech.github.io/ink/ink_env/fn.terminate_contract.html).
    /// In such a case, the condition involving the contract balance will work incorrectly, what
    /// may lead to security issues, including DoS attacks and draining contract's gas.
    ///
    /// **Known problems**: There are many ways to implement comparison between integers in Rust.
    /// This lint is not trying to be exhaustive; instead, it addresses the most common cases that
    /// will occur in almost all real-world contracts.
    ///
    /// **Example:**
    ///
    /// Assume, there is an attacker contract that sends all its funds to the target contract when
    /// terminated:
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
}

/// TransferFunction is a temporary object used by the implementation of transfer function
/// to iterate over MIR statements for a single function.
struct TransferFunction<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    state: &'a mut BitSet<Local>,
}

impl<'a, 'tcx> StrictBalanceEqualityAnalysis<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'tcx>) -> Self {
        Self { cx }
    }
}

impl<'a, 'tcx> AnalysisDomain<'tcx> for StrictBalanceEqualityAnalysis<'a, 'tcx> {
    /// A lattice that represents program's state. `BitSet` is a powerset over MIR Locals
    /// defined in the analyzed function. Inclusion to the set means that the Local is
    /// tainted with some operation with `self.env().balance()`.
    type Domain = BitSet<Local>;

    const NAME: &'static str = "strict_balance_equality";

    type Direction = Forward;

    fn bottom_value(&self, body: &Body) -> Self::Domain {
        // bottom = no balance taints
        BitSet::new_empty(body.local_decls().len())
    }

    fn initialize_start_block(&self, _body: &Body, _state: &mut Self::Domain) {
        // Source of taints are: locals, contract fields and mutable arguments.
        // TODO: No of these are tainted with balance at the beginning, but we should fix
        // it when working on interprocedural analysis.
    }
}

/// The implementation of the transfer function for the dataflow problem
impl<'a, 'tcx> Analysis<'tcx> for StrictBalanceEqualityAnalysis<'a, 'tcx> {
    fn apply_statement_effect(
        &mut self,
        state: &mut Self::Domain,
        statement: &Statement,
        location: Location,
    ) {
        TransferFunction { cx: self.cx, state }.visit_statement(statement, location);
    }

    fn apply_terminator_effect(
        &mut self,
        state: &mut Self::Domain,
        terminator: &Terminator,
        location: Location,
    ) {
        TransferFunction { cx: self.cx, state }.visit_terminator(terminator, location);
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

/// MIR visitor that iterates over statements of a function
impl Visitor<'_> for TransferFunction<'_, '_> {
    fn visit_assign(&mut self, place: &Place, rvalue: &Rvalue, _: Location) {
        // Direct comparison
        if let Rvalue::BinaryOp(BinOp::Eq | BinOp::Ne, box (lhs, rhs)) = rvalue {
            if tainted_with_balance(self.state, lhs).is_some()
                || tainted_with_balance(self.state, rhs).is_some()
            {
                self.state.insert(place.local);
            }
        }
    }

    fn visit_terminator(&mut self, terminator: &Terminator, _: Location) {
        if_chain! {
            if let TerminatorKind::Call { func, destination, .. } = &terminator.kind;
            if let Some((fn_def_id, _)) = func.const_fn_def();
            if match_def_path(self.cx, fn_def_id, &["ink", "env_access", "EnvAccess", "balance"]);
            then {
                self.state.insert(destination.local);
            }
        }
    }
}

/// Returns Local if the given operand is tainted with balance in the `state` lattice
fn tainted_with_balance(state: &BitSet<Local>, op: &Operand) -> Option<Local> {
    if_chain! {
        if let Some(place) = op.place();
        if state.contains(place.local);
        then { Some(place.local) } else { None }
    }
}

impl<'tcx> LateLintPass<'tcx> for StrictBalanceEquality {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: FnKind<'tcx>,
        _decl: &'tcx FnDecl<'_>,
        _body: &'tcx hir::Body<'tcx>,
        _span: Span,
        fn_def_id: LocalDefId,
    ) {
        let fn_mir = cx.tcx.optimized_mir(fn_def_id);
        let mut taint_results = StrictBalanceEqualityAnalysis::new(cx)
            .into_engine(cx.tcx, fn_mir)
            .iterate_to_fixpoint()
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
                then {
                    let span = terminator.source_info.span;
                    span_lint_and_then(
                        cx,
                        STRICT_BALANCE_EQUALITY,
                        span,
                        "dangerous strict balance equality",
                        |diag| {
                            let snippet = snippet_opt(cx, span).expect("snippet must exist");
                            diag.span_suggestion(
                                span,
                                "consider using non-strict equality operators instead: `<`, `>`".to_string(),
                                snippet,
                                Applicability::Unspecified,
                            );
                        },
                    )
                }
            }
        }
    }
}
