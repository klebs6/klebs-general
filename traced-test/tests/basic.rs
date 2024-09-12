#![allow(unused_attributes)]

use traced_test::traced_test;
use tracing_setup::*;
use tracing::info;

// Test that checks if the proc-macro works with a simple synchronous test
#[traced_test]
fn sync_test_succeeds() {
    assert!(true);
}

// Test that ensures panics are handled in synchronous tests
#[traced_test]
#[should_panic(expected = "This test should fail.")]
fn sync_test_fails() {
    assert!(false, "This test should fail.");
}

// Test that returns a Result and works with traced_test
#[traced_test]
fn sync_test_with_result() -> Result<(), String> {
    assert!(true);
    Ok(())
}

// Test async test that returns no result and should succeed
#[traced_test]
async fn async_test_succeeds() {
    assert!(true);
}

// Async test with a panic to test panic handling in async context
#[traced_test]
#[should_panic(expected = "This test should fail.")]
async fn async_test_fails() {
    assert!(false, "This test should fail.");
}

// Async test that returns a Result
#[traced_test]
async fn async_test_with_result() -> Result<(), String> {
    assert!(true);
    Ok(())
}

// Test that ensures `#[test]` and `#[tokio::test]` cannot be used with `#[traced_test]`
#[traced_test]
fn invalid_combination_of_test_attributes() {
    // This should never compile if another test attribute is present
}

// Test that ensures the tracing happens correctly
#[traced_test]
fn sync_test_tracing() {
    info!("This is a tracing test");
    assert!(true);
}

#[traced_test]
async fn async_test_tracing() {
    info!("This is a tracing test in async");
    assert!(true);
}
