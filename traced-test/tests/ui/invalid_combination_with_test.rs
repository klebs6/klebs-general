use traced_test::traced_test;

#[traced_test]
#[test]
fn invalid_combination_of_test_attributes_with_test() {
    // This should never compile if another test attribute is present.
    // Test will fail if `#[test]` is present alongside `#[traced_test]`.
}

