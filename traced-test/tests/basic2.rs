// ---------------- [ File: tests/basic2.rs ]
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
    #[traced_test(
        should_fail(message = "Async test failed")
    )]
    async fn async_test_with_result_failure() 
        -> Result<(), String> 
    {
        info!("we should not see this log message because the test is expected to fail with the provided message");
        Err("Async test failed".to_string())
    }

    #[traced_test]
    fn test_sync_pass_no_should_fail() {
        info!("This log should not be displayed because the test passes.");
        assert!(true);
    }
}
