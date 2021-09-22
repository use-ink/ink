// Copyright 2018-2021 Parity Technologies (UK) Ltd.
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
fn compile_tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/ui/contract/pass/01-noop-contract.rs");
    t.pass("tests/ui/contract/pass/02-flipper-contract.rs");
    t.pass("tests/ui/contract/pass/03-incrementer-contract.rs");
    t.pass("tests/ui/contract/pass/04-erc20-contract.rs");
    t.pass("tests/ui/contract/pass/05-erc721-contract.rs");
    t.pass("tests/ui/contract/pass/06-non-ink-items.rs");
    t.pass("tests/ui/contract/pass/07-flipper-as-dependency.rs");
    if option_env!("INK_COVERAGE_REPORTING") != Some("true") {
        // The cross-calling implementation for traits provides
        // an invalid implementation for non-valid message calls
        // (e.g. cross-calling a `mut` message from a non-`mut` message).
        // So calling those will result in a compiler or linker error.
        //
        // The coverage reporting CI stage though also links dead code,
        // hence resulting in this invalid implementation being linked
        // and thus a linker error.
        t.pass("tests/ui/contract/pass/08-flipper-as-dependency-trait.rs");
    }
    t.pass("tests/ui/contract/pass/09-static-env.rs");
    t.pass("tests/ui/contract/pass/10-derive-for-storage.rs");
    t.pass("tests/ui/contract/pass/11-alias-storage-struct-impl.rs");

    t.compile_fail("tests/ui/contract/fail/C-00-constructor-self-ref.rs");
    t.compile_fail("tests/ui/contract/fail/C-01-constructor-self-mut.rs");
    t.compile_fail("tests/ui/contract/fail/C-02-constructor-self-val.rs");
    t.compile_fail("tests/ui/contract/fail/C-03-constructor-missing-return.rs");
    t.compile_fail("tests/ui/contract/fail/C-04-missing-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-10-async-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-11-unsafe-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-12-const-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-13-abi-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-14-payable-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-15-payable-trait-constructor.rs");
    t.compile_fail("tests/ui/contract/fail/C-16-function-arg-struct-destructuring.rs");

    t.compile_fail("tests/ui/contract/fail/H-01-invalid-dyn-alloc.rs");
    t.compile_fail("tests/ui/contract/fail/H-02-invalid-as-dependency.rs");
    t.compile_fail("tests/ui/contract/fail/H-03-use-forbidden-idents.rs");

    t.compile_fail("tests/ui/contract/fail/M-01-missing-message.rs");
    t.compile_fail("tests/ui/contract/fail/M-02-message-missing-self-arg.rs");
    t.compile_fail("tests/ui/contract/fail/M-03-message-returns-self.rs");
    t.compile_fail("tests/ui/contract/fail/M-04-message-returns-non-codec.rs");
    t.compile_fail("tests/ui/contract/fail/M-05-message-invalid-selector.rs");
    t.compile_fail("tests/ui/contract/fail/M-06-message-invalid-selector-type.rs");
    t.compile_fail("tests/ui/contract/fail/M-10-method-unknown-ink-marker.rs");

    t.compile_fail("tests/ui/contract/fail/S-01-missing-storage-struct.rs");
    t.compile_fail("tests/ui/contract/fail/S-02-multiple-storage-structs.rs");
    t.compile_fail("tests/ui/contract/fail/S-03-struct-unknown-ink-marker.rs");
    t.compile_fail("tests/ui/contract/fail/S-04-non-storage-ink-impls.rs");
    t.compile_fail("tests/ui/contract/fail/S-05-storage-as-event.rs");
    t.compile_fail("tests/ui/contract/fail/S-06-event-as-storage.rs");

    t.compile_fail("tests/ui/contract/fail/N-01-namespace-invalid-identifier.rs");
    t.compile_fail("tests/ui/contract/fail/N-02-namespace-invalid-type.rs");
    t.compile_fail("tests/ui/contract/fail/N-03-namespace-missing-argument.rs");

    t.pass("tests/ui/chain_extension/E-01-simple.rs");
}
