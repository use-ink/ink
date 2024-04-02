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

#[test]
fn ui_tests_blake2b_pass() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/blake2b/pass/*.rs");
}

#[test]
fn ui_tests_blake2b_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/blake2b/fail/*.rs");
}

#[test]
fn ui_tests_selector_id_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/selector_id/pass/*.rs");
}

#[test]
fn ui_tests_selector_id_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/selector_id/fail/*.rs");
}

#[test]
fn ui_tests_selector_bytes_pass() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/selector_bytes/pass/*.rs");
}

#[test]
fn ui_tests_selector_bytes_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/selector_bytes/fail/*.rs");
}

#[test]
fn ui_tests_contract_pass() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/contract/pass/*.rs");
}

#[test]
fn ui_tests_contract_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/contract/fail/*.rs");
}

#[test]
fn ui_tests_event_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/event/pass/*.rs");
}

#[test]
fn ui_tests_event_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/event/fail/*.rs");
}

#[test]
fn ui_tests_storage_item_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/storage_item/pass/*.rs");
}

#[test]
fn ui_tests_storage_item_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/storage_item/fail/*.rs");
}

#[test]
fn ui_tests_trait_def_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/trait_def/pass/*.rs");
}

#[test]
fn ui_tests_trait_def_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/trait_def/fail/*.rs");
}

#[test]
fn ui_tests_chain_extension_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/chain_extension/E-01-simple.rs");
}

#[test]
fn ui_tests_pay_with_call_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pay_with_call/pass/multiple_args.rs");
}

#[test]
fn ui_tests_scale_derive_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/scale_derive/pass/*.rs");
}

#[test]
fn ui_tests_scale_derive_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/scale_derive/fail/*.rs");
}
