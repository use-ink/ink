#[test]
fn compile_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass/01-noop-contract.rs");
    t.pass("tests/ui/pass/02-multiple-constructors.rs");
    t.pass("tests/ui/pass/03-flipper-contract.rs");
    t.compile_fail("tests/ui/fail/01-constructor-returns.rs");
    t.compile_fail("tests/ui/fail/02-missing-constructor.rs");
}
