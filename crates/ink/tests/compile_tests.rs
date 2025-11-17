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

use std::{
    env,
    process::{
        Command,
        ExitStatus,
    },
};

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
