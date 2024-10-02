#![allow(unused_attributes)]
#![allow(unused_imports)]
#![allow(non_snake_case)]

use disable_macro::disable;

use traced_test::*;
use tracing_setup::*;
use tracing::info;

use std::time::Duration;
use tokio::time::sleep;

mod good {

    use super::*;

    // Async test that returns a Result (failure case)
    #[traced_test]
    #[should_fail(message = "Async test failed")]
    async fn async_test_with_result_failure() 
        -> Result<(), String> 
    {
        info!("we should not see this log message because the test is expected to fail with the provided message");
        Err("Async test failed".to_string())
    }

    // Test that returns a Result and works with traced_test
    // (failure case)
    //
    // Overall, the test should not crash our test suite because
    // the error message matches the expected failure message
    //
    #[traced_test]
    #[should_fail(message = "Test failed")]
    fn sync_test_with_result_failure() -> Result<(), String> {
        info!("we should not see this log message because the test is expected to fail with the provided message");
        Err("Test failed".to_string())
    }

    // Test that checks if the proc-macro works with a simple
    // synchronous test
    //
    #[traced_test]
    fn sync_test_succeeds() {
        info!("we should not see this log message because the test passes. we don't need to see logs when a test passes");
        assert!(true, "This test should pass.");
    }

    #[traced_test]
    fn check_fundamental_behavior() -> Result<(),String> {
        debug!("we don't need to see logs when a test passes");
        assert!(true, "This test should pass.");
        //Err("we should see the logs if we uncomment this line and return the error".into())
        Ok(())
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
    //#[disable]
    #[traced_test]
    #[should_fail(message = "This test should fail.")]
    fn sync_test_fails() {
        info!("we should not see this log message because the test is expected to fail with the provided message");
        assert!(false, "This test should fail.");
    }

    // Test that returns a Result and works with traced_test
    // (success case)
    //
    #[traced_test]
    fn sync_test_with_result_success() -> Result<(), String> {
        info!("we should not see this log message because the test passes");
        assert!(true, "This test should pass.");
        Ok(())
    }

    // Test async test that returns no result and should succeed
    //
    #[traced_test]
    async fn async_test_succeeds() {
        info!("we should not see this log message because the test passes");
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
        info!("we should not see this log message because the test is expected to fail with the provided message");
        assert!(false, "This test should fail.");
    }

    // Async test that returns a Result (success case)
    //
    #[traced_test]
    async fn async_test_with_result_success() -> Result<(), String> {
        info!("we should not see this log message because the test passes");
        assert!(true, "Async test should pass.");
        Ok(())
    }

    // Test that ensures the tracing happens correctly in a synchronous test
    #[traced_test]
    fn sync_test_tracing() {
        info!("we should not see this log message because th test passes");
        assert!(true, "Tracing should be captured.");
    }

    // Test that ensures the tracing happens correctly in an asynchronous test
    #[traced_test]
    async fn async_test_tracing() {
        info!("we should not see this log message because the test passes");
        assert!(true, "Async tracing should be captured.");
    }

    //TODO: the assertion does not trigger the proper logging!
    //
    #[traced_test]
    #[should_fail(message = "sync test failure")]
    fn sync_test_failure_but_proper_logging() {
        info!("we should not see this log message because we expect the test to fail with the provided message");
        assert!(false, "sync test failure");
    }


    // TODO: the assertion does not trigger the logging!
    //
    // Test that ensures the tracing happens correctly in an asynchronous test
    #[traced_test]
    #[should_fail(message = "async test failure")]
    async fn async_test_failure_but_proper_logging() {
        info!("we should not see this log message because the test fails in the way that we expect");
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
        info!("we should not see this log message because the test passes");
        let result = real_async_function_success().await?;
        assert_eq!(result, 42, "Expected 42 from real_async_function_success");
        Ok(())
    }

    // Test that ensures async function with await and result handling works (failure case)
    #[traced_test]
    async fn async_test_real_async_function_failure() -> Result<(), String> {
        info!("we should not see this log message because overall, the test passes");
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

    #[traced_test]
    fn test_sync_pass_no_should_fail() {
        info!("This log should not be displayed because the test passes.");
        assert!(true);
    }

    #[traced_test]
    #[should_fail(message = "Expected failure")]
    fn test_sync_fail_with_matching_should_fail() {
        info!("This log should not be displayed because the test fails as expected.");
        panic!("Expected failure");
    }

    #[traced_test]
    fn test_no_trace_on_success() {
        info!("This log should not be displayed unless the test fails.");
        assert!(true);
    }
}

// during testing, we can re-enable these tests just to make sure we get the
// expected failures we anticipate
//
#[disable]
mod expect_failure {
    use super::*;

    #[traced_test]
    fn EXPECT_FAILURE_sync_test_failure_assertion_should_trigger_logging() {
        info!("EXPECT_FAILURE_sync_test_failure_assertion_should_trigger_logging -- This log should be displayed because the test fails unexpectedly.");
        assert!(false, "This test should fail.");
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
    fn EXPECT_FAILURE_sync_test_failure_assertion_must_trigger_logging() {
        info!("EXPECT_FAILURE_sync_test_failure_assertion_must_trigger_logging -- we must see this log message because the test fails with a false assertion");
        assert!(false, "This test should fail.");
    }

    #[traced_test]
    async fn EXPECT_FAILURE_async_test_uncaught_failure() {
        info!("EXPECT_FAILURE_async_test_uncaught_failure -- we should see this log message because the test fails");
        assert!(false, "This test should fail.");
    }

    #[traced_test]
    #[should_fail(message = "This test should fail.")]
    async fn EXPECT_FAILURE_async_test_failure_with_unexpected_error_message() {
        info!("EXPECT_FAILURE_async_test_failure_with_unexpected_error_message -- we should see this log message because the test fails with a message we did not expect");
        assert!(false, "unexpected error message");
    }

    #[traced_test]
    fn EXPECT_FAILURE_sync_test_failure_we_need_to_see_the_tracing() {
        info!("EXPECT_FAILURE_sync_test_failure_we_need_to_see_the_tracing -- we must see this log message because the test fails unexpectedly");
        assert!(false, "sync test failure");
    }

    #[traced_test]
    async fn EXPECT_FAILURE_async_test_failure_we_need_to_see_the_tracing() {
        info!("EXPECT_FAILURE_async_test_failure_we_need_to_see_the_tracing -- we must see this log message because the test fails unexpectedly");
        assert!(false, "async test failure");
    }

    // Test with async panic after a delay to ensure delayed panics are caught
    #[traced_test]
    async fn EXPECT_FAILURE_async_test_with_delayed_panic() {
        info!("EXPECT_FAILURE_async_test_with_delayed_panic -- Test started, waiting for async operation... (we should see this log message because an unexpected panic occurs)");
        sleep(Duration::from_millis(50)).await; // Simulate async delay
        info!("EXPECT_FAILURE_async_test_with_delayed_panic -- Async operation completed... (we should see this log message because an unexpected panic occurs)");
        panic!("Delayed panic occurred.");
    }

    #[traced_test]
    fn EXPECT_FAILURE_test_sync_fail_no_should_fail() {
        info!("EXPECT_FAILURE_test_sync_fail_no_should_fail -- This log should be displayed because the test fails unexpectedly.");
        assert!(false, "Unexpected failure");
    }

    #[traced_test]
    #[should_fail(message = "Expected failure")]
    fn EXPECT_FAILURE_test_sync_pass_with_should_fail() {
        info!("EXPECT_FAILURE_test_sync_pass_with_should_fail -- This log should be displayed because the test passes unexpectedly.");
        assert!(true);
    }

    #[traced_test]
    #[should_fail(message = "Expected failure")]
    fn EXPECT_FAILURE_test_sync_fail_with_non_matching_should_fail() {
        info!("EXPECT_FAILURE_test_sync_fail_with_non_matching_should_fail -- This log should be displayed because the test fails with an unexpected message.");
        panic!("Different failure message");
    }
}
