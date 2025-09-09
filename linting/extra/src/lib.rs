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

dylint_linting::dylint_library!();

extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_mir_dataflow;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_type_ir;

mod non_fallible_api;
mod primitive_topic;
mod storage_never_freed;
mod strict_balance_equality;

#[doc(hidden)]
#[unsafe(no_mangle)]
pub fn register_lints(
    _sess: &rustc_session::Session,
    lint_store: &mut rustc_lint::LintStore,
) {
    lint_store.register_lints(&[
        primitive_topic::PRIMITIVE_TOPIC,
        storage_never_freed::STORAGE_NEVER_FREED,
        strict_balance_equality::STRICT_BALANCE_EQUALITY,
        non_fallible_api::NON_FALLIBLE_API,
    ]);
    lint_store.register_late_pass(|_| Box::new(primitive_topic::PrimitiveTopic));
    lint_store.register_late_pass(|_| Box::new(storage_never_freed::StorageNeverFreed));
    lint_store
        .register_late_pass(|_| Box::new(strict_balance_equality::StrictBalanceEquality));
    lint_store.register_late_pass(|_| Box::new(non_fallible_api::NonFallibleAPI));
}

#[test]
fn ui() {
    dylint_testing::ui_test_examples(env!("CARGO_PKG_NAME"));
}
