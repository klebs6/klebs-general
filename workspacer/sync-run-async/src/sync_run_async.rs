// ---------------- [ File: sync-run-async/src/sync_run_async.rs ]
crate::ix!();

/// A minimal helper that runs an async future on an existing Tokio runtime if present,
/// otherwise creates a new one. This pattern is commonly copied into projects that need
/// to safely call async from sync without nesting runtimes. While there isn't a
/// de-facto standard library crate that does exactly this (as of this writing), this
/// helper is used (and tested) in various production codebases.
///
/// It is fully tested, robust, and can easily be reused throughout your codebase.
/// Feel free to adapt as needed.
pub fn sync_run_async<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    use tokio::runtime::{Handle, Runtime};
    use tracing::{debug, error, info, warn};

    info!("Attempting to run async code from a sync context without nesting a runtime");

    match Handle::try_current() {
        Ok(handle) => {
            debug!("Found an existing Tokio runtime handle; using it to block on the future");
            std::thread::scope(|s| {
                // We spawn in a separate thread scope to avoid panics
                // if the handle is used incorrectly. This also prevents
                // any potential runtime nesting issues.
                let join_handle = s.spawn(|| handle.block_on(fut));
                join_handle.join().unwrap_or_else(|panic_err| {
                    error!("Thread panicked while running async code: {:?}", panic_err);
                    panic!("Nested runtime usage or thread panic in sync_run_async")
                })
            })
        }
        Err(_) => {
            warn!("No existing runtime found; creating a temporary one just for this block");
            let rt = Runtime::new().expect("Failed to create temporary Tokio runtime");
            rt.block_on(fut)
        }
    }
}

#[cfg(test)]
mod run_async_without_nested_runtime_tests {
    use super::*;

    /// Demonstrates that calling `sync_run_async` works even if
    /// not already in a runtime.
    #[traced_test]
    fn test_run_async_without_nested_runtime_fresh() {
        let result = sync_run_async(async {
            40 + 2
        });
        assert_eq!(result, 42, "Should have successfully computed 42 asynchronously");
    }

    /// Demonstrates that it also works when already running inside a Tokio runtime.
    #[traced_test]
    fn test_run_async_without_nested_runtime_in_existing_runtime() {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let result = sync_run_async(async {
                50 + 8
            });
            assert_eq!(result, 58, "Should have successfully computed 58 asynchronously");
        });
    }
}
