// Copyright 2018-2022 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use clippy_utils::{
    diagnostics::span_lint_and_then,
    source::snippet_opt,
};
use if_chain::if_chain;
use regex::Regex;
use rustc_ast as ast;
use rustc_errors::Applicability;
use rustc_hir::{
    def_id::DefId,
    intravisit::{
        walk_fn,
        walk_item,
        walk_qpath,
        FnKind,
        Visitor,
    },
    BodyId,
    FnDecl,
    HirId,
    Item,
    ItemKind,
    QPath,
    VariantData,
};
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_middle::hir::nested_filter;
use rustc_session::{
    declare_lint,
    declare_lint_pass,
};
use rustc_span::{
    source_map::Span,
    symbol::sym,
};

declare_lint! {
    /// **What it does:** Checks for ink! contracts that use
    /// [`ink_storage::Mapping`](https://paritytech.github.io/ink/ink_storage/struct.Mapping.html)
    /// in their storage without initializing it properly in the ink! constructor.
    ///
    /// **Why is this bad?** If `Mapping` is not properly initialized corruption of storage
    /// might occur.
    ///
    /// **Known problems:** The lint currently requires for the
    /// `ink_lang::utils::initialize_contract(…)` function call to be made on the top
    /// level of the constructor *and* it has be made explicitly in this form.
    ///
    /// The lint can currently not detect if `initialize_contract` would be called in a
    /// nested function within the constructor.
    ///
    /// **Example:**
    ///
    /// ```rust
    /// // Good
    /// use ink_storage::{traits::SpreadAllocate, Mapping};
    ///
    /// #[ink(storage)]
    /// #[derive(SpreadAllocate)]
    /// pub struct MyContract {
    ///     balances: Mapping<AccountId, Balance>,
    /// }
    ///
    /// #[ink(constructor)]
    /// pub fn new() -> Self {
    ///     ink_lang::utils::initialize_contract(Self::new_init)
    /// }
    ///
    /// /// Default initializes the contract.
    /// fn new_init(&mut self) {
    ///   let caller = Self::env().caller();
    ///   let value: Balance = Default::default();
    ///   self.balances.insert(&caller, &value);
    /// }
    /// ```
    ///
    /// ```rust
    /// // Bad
    /// use ink_storage::{traits::SpreadAllocate, Mapping};
    ///
    /// #[ink(storage)]
    /// #[derive(SpreadAllocate)]
    /// pub struct MyContract {
    ///     balances: Mapping<AccountId, Balance>,
    /// }
    ///
    /// #[ink(constructor)]
    /// pub fn new() -> Self {
    ///     Self {
    ///         balances: Default::default(),
    ///     }
    /// }
    /// ```
    pub MAPPING_INITIALIZED,
    Deny,
    "Error on ink! contracts that use `ink_storage::Mapping` without initializing it."
}

declare_lint_pass!(MappingInitialized => [MAPPING_INITIALIZED]);

/// An ink! attribute.
#[derive(PartialEq)]
enum InkAttribute {
    // #[ink(storage)]
    Storage,
    // #[ink(constructor)]
    Constructor,
}

/// Returns `Some(InkAttribute)` if an ink! attribute is among `attributes`.
fn get_ink_attribute(attributes: &[ast::Attribute]) -> Option<InkAttribute> {
    const INK_STORAGE: &str = "__ink_dylint_Storage";
    const INK_CONSTRUCTOR: &str = "__ink_dylint_Constructor";

    let attrs = format!("symbol: \"{:?}\"", attributes);
    if attrs.contains(INK_STORAGE) {
        Some(InkAttribute::Storage)
    } else if attrs.contains(INK_CONSTRUCTOR) {
        Some(InkAttribute::Constructor)
    } else {
        None
    }
}

impl<'tcx> LateLintPass<'tcx> for MappingInitialized {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'_>) {
        let attrs = cx.tcx.hir().attrs(item.hir_id());
        let ink_attrs = get_ink_attribute(attrs);
        if ink_attrs.is_none() {
            return
        }

        if let ItemKind::Struct(variant_data, _) = &item.kind {
            if let VariantData::Unit(..) = variant_data {
                return
            }
            check_struct(cx, item, variant_data);
        }
    }
}

/// Examines a `struct`. If the struct is annotated as an ink! storage struct
/// we examine if there is a `ink_storage::Mapping` in it; in case there
/// is we continue checking all ink! constructors for correct usage of the
/// `ink_lang::utils::initialize_contract` API.
///
/// This function is backwards compatible, in case no annotated ink! struct is
/// found nothing happens.
fn check_struct<'a>(cx: &LateContext<'a>, item: &'a Item, data: &VariantData) {
    let attrs = cx.tcx.hir().attrs(item.hir_id());
    match get_ink_attribute(attrs) {
        Some(InkAttribute::Storage) => {}
        _ => return,
    }

    let mut marker = None;
    let fields = data.fields();
    let storage_contains_mapping = fields.iter().any(|field| {
        let re = Regex::new(r"ink_storage\[.*\]::lazy::mapping::Mapping")
            .expect("failed creating regex");
        if re.is_match(&format!("{:?}", field.ty.kind)) {
            marker = Some(field);
            return true
        }
        false
    });

    if !storage_contains_mapping {
        log::debug!("Found `#[ink(storage)]` struct without `Mapping`");
        return
    }
    log::debug!("Found `#[ink(storage)]` struct with `Mapping`");

    let inherent_impls = cx.tcx.inherent_impls(item.def_id);
    let constructors_without_initialize: Vec<Span> = inherent_impls
        .iter()
        .map(|imp_did| item_from_def_id(cx, *imp_did))
        .filter_map(|item| constructor_but_no_initialize(cx, item))
        .collect();
    log::debug!(
        "Result of searching for constructors without initialize: {:?}",
        constructors_without_initialize
    );

    if_chain! {
        if storage_contains_mapping;
        if !constructors_without_initialize.is_empty();
        then {
            let constructor_span = constructors_without_initialize.get(0)
                .expect("at least one faulty constructor must exist");
            let snippet = snippet_opt(cx, *constructor_span)
                .expect("snippet must exist");
            span_lint_and_then(
                cx,
                MAPPING_INITIALIZED,
                item.span,
                &format!(
                    "`#[ink(storage)]` on `{}` contains `ink_storage::Mapping` without initializing it in the contract constructor.",
                    item.ident
                ),
                |diag| {
                    diag.span_suggestion(
                        *constructor_span,
                        "add an `ink_lang::utils::initialize_contract(…)` function in this constructor",
                        snippet,
                        Applicability::Unspecified,
                    );
                    diag.span_help(marker.expect("marker must exist").span, "this field uses `ink_storage::Mapping`");
                });
        }
    }
}

/// Returns `Some(span)` if a constructor without a call of
/// `ink_lang::utils::initialize_contract(…)` was found.
fn constructor_but_no_initialize<'tcx>(
    cx: &LateContext<'tcx>,
    item: &'tcx Item<'_>,
) -> Option<Span> {
    let mut visitor = InkAttributeVisitor {
        cx,
        ink_attribute: None,
        constructor_info: None,
    };

    walk_item(&mut visitor, item);

    match visitor.constructor_info {
        Some(info) if !info.uses_initialize_contract => Some(info.span),
        _ => None,
    }
}

/// Visitor for ink! attributes.
struct InkAttributeVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    ink_attribute: Option<InkAttribute>,
    constructor_info: Option<InkConstructor>,
}

// Information about an ink! constructor.
struct InkConstructor {
    // The constructor has a call to `ink_lang::utils::initialize_contract(…)`
    // in its function.
    uses_initialize_contract: bool,
    // The span for the constructor function.
    span: Span,
}

impl<'tcx> Visitor<'tcx> for InkAttributeVisitor<'_, 'tcx> {
    type NestedFilter = nested_filter::All;

    fn visit_fn(
        &mut self,
        kind: FnKind<'tcx>,
        decl: &'tcx FnDecl<'_>,
        body_id: BodyId,
        span: Span,
        id: HirId,
    ) {
        // We can return immediately if an incorrect constructor was already found
        if let Some(constructor) = &self.constructor_info {
            if !constructor.uses_initialize_contract {
                return
            }
        }

        let attrs: Vec<ast::Attribute> = self
            .cx
            .tcx
            .get_attrs(id.owner.to_def_id(), sym::cfg)
            .cloned()
            .collect();
        self.ink_attribute = get_ink_attribute(&attrs);

        if self.ink_attribute == Some(InkAttribute::Storage) {
            return
        } else if self.ink_attribute == Some(InkAttribute::Constructor) {
            log::debug!(
                "Found constructor, starting to search for `initialize_contract`"
            );
            let mut visitor = InitializeContractVisitor {
                cx: self.cx,
                uses_initialize_contract: false,
            };
            walk_fn(&mut visitor, kind, decl, body_id, span, id);

            log::debug!(
                "Has `initialize_contract`? {:?}",
                visitor.uses_initialize_contract
            );
            self.constructor_info = Some(InkConstructor {
                uses_initialize_contract: visitor.uses_initialize_contract,
                span,
            });

            return
        }

        walk_fn(self, kind, decl, body_id, span, id);
    }

    fn nested_visit_map(&mut self) -> Self::Map {
        self.cx.tcx.hir()
    }
}

/// Visitor to determine if a `fn` contains a call to `ink_lang::utils::initialize_contract(…)`.
///
/// # Known Limitation
///
/// This function currently only finds call to `initialize_contract` which happen
/// on the first level of the `fn` ‒ no nested calls are found! So if you would
/// call `initialize_contract` within a sub-function of the ink! constructor
/// this is not recognized!
struct InitializeContractVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    uses_initialize_contract: bool,
}

impl<'tcx> Visitor<'tcx> for InitializeContractVisitor<'_, 'tcx> {
    type NestedFilter = nested_filter::All;

    fn visit_qpath(&mut self, qpath: &'tcx QPath<'_>, id: HirId, span: Span) {
        log::debug!("Visiting path {:?}", qpath);
        if self.uses_initialize_contract {
            return
        }

        if let QPath::Resolved(_, path) = qpath {
            log::debug!("QPath: {:?}", path.res);
            let re = Regex::new(
                r"ink_lang\[.*\]::codegen::dispatch::execution::initialize_contract",
            )
            .expect("failed creating regex");
            if re.is_match(&format!("{:?}", path.res)) {
                self.uses_initialize_contract = true;
                return
            }
        }

        walk_qpath(self, qpath, id, span);
    }

    fn nested_visit_map(&mut self) -> Self::Map {
        self.cx.tcx.hir()
    }
}

/// Returns the `rustc_hir::Item` for a `rustc_hir::def_id::DefId`.
fn item_from_def_id<'tcx>(cx: &LateContext<'tcx>, def_id: DefId) -> &'tcx Item<'tcx> {
    cx.tcx.hir().expect_item(def_id.expect_local())
}
