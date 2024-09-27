use traced_test::traced_test;

#[traced_test]
#[tokio::test]
async fn invalid_combination_of_test_attributes_with_tokio_test() {
    // This should never compile if another test attribute is present.
    // Test will fail if `#[tokio::test]` is present alongside `#[traced_test]`.
}

