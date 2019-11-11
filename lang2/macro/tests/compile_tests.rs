// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of ink!.
//
// ink! is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// ink! is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with ink!.  If not, see <http://www.gnu.org/licenses/>.

#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/01-noop-contract.rs");
    t.pass("tests/ui/pass/02-flipper-contract.rs");
    t.pass("tests/ui/pass/03-incrementer-contract.rs");
    t.pass("tests/ui/pass/04-erc20-contract.rs");
    t.pass("tests/ui/pass/05-erc721-contract.rs");
    t.pass("tests/ui/pass/06-non-ink-items.rs");
    t.pass("tests/ui/pass/07-flipper-as-dependency.rs");
    t.compile_fail("tests/ui/fail/01-constructor-returns.rs");
    t.compile_fail("tests/ui/fail/02-missing-constructor.rs");
    t.compile_fail("tests/ui/fail/03-invalid-version.rs");
    t.compile_fail("tests/ui/fail/04-missing-message.rs");
    t.compile_fail("tests/ui/fail/05-forbidden-idents.rs");
    t.compile_fail("tests/ui/fail/06-access-generated-fields.rs");
    t.compile_fail("tests/ui/fail/07-constructor-missing-self.rs");
    t.compile_fail("tests/ui/fail/08-constructor-self-ref.rs");
    t.compile_fail("tests/ui/fail/09-constructor-self-val.rs");
    t.compile_fail("tests/ui/fail/10-async-constructor.rs");
    t.compile_fail("tests/ui/fail/11-unsafe-constructor.rs");
    t.compile_fail("tests/ui/fail/12-const-constructor.rs");
    t.compile_fail("tests/ui/fail/13-abi-constructor.rs");
    t.compile_fail("tests/ui/fail/14-missing-storage-struct.rs");
    t.compile_fail("tests/ui/fail/15-multiple-storage-structs.rs");
    t.compile_fail("tests/ui/fail/16-storage-impl-ident-conflict.rs");
    t.compile_fail("tests/ui/fail/17-conflicting-ink-markers.rs");
    t.compile_fail("tests/ui/fail/18-conflicting-ink-markers-2.rs");
    t.compile_fail("tests/ui/fail/19-unknown-struct-ink-marker.rs");
    t.compile_fail("tests/ui/fail/20-unknown-method-marker.rs");
}
