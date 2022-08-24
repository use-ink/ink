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

#![feature(rustc_private)]

dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod mapping_initialized;

#[doc(hidden)]
#[no_mangle]
pub fn register_lints(
    _sess: &rustc_session::Session,
    lint_store: &mut rustc_lint::LintStore,
) {
    lint_store.register_lints(&[mapping_initialized::MAPPING_INITIALIZED]);
    lint_store.register_late_pass(|| Box::new(mapping_initialized::MappingInitialized));
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
