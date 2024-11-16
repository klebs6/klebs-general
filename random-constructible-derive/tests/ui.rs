#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/01-simple-enum.rs");
    t.compile_fail("tests/ui/02-non-unit-variants.rs");
    t.pass("tests/ui/03-default-probabilities.rs");
}

