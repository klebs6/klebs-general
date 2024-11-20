#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-simple-enum.rs");
    t.pass("tests/ui/02-non-unit-variants-enum.rs");
    t.pass("tests/ui/02-non-unit-variants-struct.rs");
    t.pass("tests/ui/02-non-unit-variants-prim.rs");
    t.pass("tests/ui/03-default-probabilities.rs");
    t.pass("tests/ui/04-env.rs");
    t.compile_fail("tests/ui/05-env-fail.rs");
}

