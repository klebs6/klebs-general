// tests/backoff_macro_tests.rs

/*
mod dummy_errors;

use backoff_macro::backoff;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::error::Error;
use dummy_errors::{DummyError, ComplexError};

#[backoff]
async fn dummy_function(attempts: Arc<AtomicUsize>) -> Result<(), ComplexError> {
    let current_attempts = attempts.fetch_add(1, Ordering::SeqCst);

    if current_attempts < 2 {
        let invalid_utf8 = vec![0, 159];
        let _ = String::from_utf8(invalid_utf8.clone()).map_err(DummyError::FromUtf8Error)?;
        Err(DummyError::FromUtf8Error(std::string::String::from_utf8(invalid_utf8).unwrap_err()).into())
    } else if current_attempts == 2 {
        Err(ComplexError::Other)
    } else {
        Ok(())
    }
}

#[tokio::test]
async fn test_backoff_macro_multiple_errors() -> Result<(), Box<dyn Error>> {
    let attempts = Arc::new(AtomicUsize::new(0));

    let result = dummy_function(attempts.clone()).await;

    match result {
        Ok(_) => println!("Function succeeded after retries"),
        Err(e) => println!("Function failed after retries: {}", e),
    }

    let attempt_count = attempts.load(Ordering::SeqCst);
    assert!(attempt_count >= 3, "The function should be retried a few times");

    Ok(())
}

#[tokio::test]
async fn test_backoff_macro() -> Result<(), Box<dyn Error>> {
    let attempts = Arc::new(AtomicUsize::new(0));

    let result = dummy_function(attempts.clone()).await;

    assert!(result.is_ok(), "The function should succeed after a few retries");

    let attempt_count = attempts.load(Ordering::SeqCst);
    assert!(attempt_count >= 3, "The function should be retried a few times");

    Ok(())
}
*/
