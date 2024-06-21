// tests/backoff_macro_tests.rs

use backoff_macro::backoff;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::error::Error;

#[backoff]
async fn dummy_function(attempts: Arc<AtomicUsize>) -> Result<(), &'static str> {

    let current_attempts = attempts.fetch_add(1, Ordering::SeqCst);

    if current_attempts < 2 {
        Err("error")
    } else {
        Ok(())
    }
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
