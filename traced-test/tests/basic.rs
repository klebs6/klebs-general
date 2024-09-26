#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use traced_test::*;
use tracing_setup::*;
use tracing::info;

use std::time::Duration;
use tokio::time::sleep;

// Test that returns a Result and works with traced_test
// (failure case)
//
// Overall, the test should not crash our test suite because
// the error message matches the expected failure message
//
#[traced_test]
#[should_fail(trace=true,message = "Test failed")]
fn sync_test_with_result_failure() -> Result<(), String> {
    info!("we should not see this log message");
    Err("Test failed".to_string())
}

// Async test that returns a Result (failure case)
#[traced_test]
#[should_fail(message = "Async test failed")]
async fn async_test_with_result_failure() -> Result<(), String> {
    info!("we should not see this log message");
    Err("Async test failed".to_string())
}

// Test that checks if the proc-macro works with a simple
// synchronous test
//
#[traced_test]
fn sync_test_succeeds() {
    info!("we should not see this log message");
    assert!(true, "This test should pass.");
}

// Test that ensures panics are handled in synchronous
// tests.
//
// The false assertion should trigger a panic which is
// caught and matched with the message in the should_fail
// attribute. 
//
// Overall, the test should not crash our test suite because
// of the matching should_fail message
//
#[traced_test]
#[should_fail(message = "This test should fail.")]
fn sync_test_fails() {
    info!("we should not see this log message");
    assert!(false, "This test should fail.");
}

// Test that returns a Result and works with traced_test
// (success case)
//
#[traced_test]
fn sync_test_with_result_success() -> Result<(), String> {
    info!("we should not see this log message");
    assert!(true, "This test should pass.");
    Ok(())
}

// Test async test that returns no result and should succeed
//
#[traced_test]
async fn async_test_succeeds() {
    info!("we should not see this log message");
    assert!(true, "Async test should pass.");
}

// Async test with a panic to test panic handling in async context
//
// Overall, the test should not crash our test suite because
// of the matching should_fail message
//
#[traced_test]
#[should_fail(message = "This test should fail.")]
async fn async_test_fails() {
    info!("we should not see this log message");
    assert!(false, "This test should fail.");
}

// Async test that returns a Result (success case)
//
#[traced_test]
async fn async_test_with_result_success() -> Result<(), String> {
    info!("we should not see this log message");
    assert!(true, "Async test should pass.");
    Ok(())
}

// Test that ensures `#[test]` and `#[tokio::test]` cannot be used with `#[traced_test]`
#[traced_test]
fn invalid_combination_of_test_attributes() {
    // This should never compile if another test attribute is present.
    // Test will fail if `#[test]` or `#[tokio::test]` is present alongside `#[traced_test]`.
}

// Test that ensures the tracing happens correctly in a synchronous test
#[traced_test]
fn sync_test_tracing() {
    info!("we should not see this log message");
    assert!(true, "Tracing should be captured.");
}

// Test that ensures the tracing happens correctly in an asynchronous test
#[traced_test]
async fn async_test_tracing() {
    info!("we should not see this log message");
    assert!(true, "Async tracing should be captured.");
}

#[traced_test]
#[should_fail(message = "sync test failure")]
fn sync_test_failure_but_proper_logging() {
    info!("we should not see this log message");
    assert!(false, "sync test failure");
}

// Test that ensures the tracing happens correctly in an asynchronous test
#[traced_test]
#[should_fail(message = "async test failure")]
async fn async_test_failure_but_proper_logging() {
    info!("we should not see this log message");
    assert!(false, "async test failure");
}

// Test that calls a real async function and awaits its result (success case)
async fn real_async_function_success() -> Result<i32, String> {
    // Simulate an async operation
    sleep(Duration::from_millis(50)).await; // Introduce a short delay
    Ok(42)
}

// Test that calls a real async function and awaits its result (failure case)
async fn real_async_function_failure() -> Result<i32, String> {
    // Simulate an async operation
    sleep(Duration::from_millis(50)).await; // Introduce a short delay
    Err("Failed async function".to_string())
}

// Test that ensures async function with await and result handling works (success case)
#[traced_test]
async fn async_test_real_async_function_success() -> Result<(), String> {
    info!("we should not see this log message");
    let result = real_async_function_success().await?;
    assert_eq!(result, 42, "Expected 42 from real_async_function_success");
    Ok(())
}

// Test that ensures async function with await and result handling works (failure case)
#[traced_test]
async fn async_test_real_async_function_failure() -> Result<(), String> {
    info!("we should not see this log message");
    let result = real_async_function_failure().await;
    assert!(result.is_err(), "Expected an error from real_async_function_failure");

    //this test passes because we successfully received an
    //error from the function we expected to fail
    Ok(())
}

// Test that checks whether async delay is correctly handled with tracing
#[traced_test]
async fn async_test_with_delay() -> Result<(), String> {
    info!("Test started, waiting for async operation... (we should not see this log message, ultimately because the overall test passes)");
    sleep(Duration::from_millis(100)).await; // Simulate async delay
    info!("Async operation completed... (we should not see this log message, ultimately because the overall test passes)");
    assert!(true, "Async test should pass after delay.");
    Ok(())
}

// Test with async panic after a delay to ensure delayed panics are caught
#[traced_test]
#[should_fail(message = "Delayed panic occurred.")]
async fn async_test_with_delayed_panic() {
    info!("Test started, waiting for async operation... (we should not see this log message, ultimately because the expected panic occurs)");
    sleep(Duration::from_millis(50)).await; // Simulate async delay
    info!("Async operation completed... (we should not see this log message, ultimately because the expected panic occurs)");
    panic!("Delayed panic occurred.");
}
