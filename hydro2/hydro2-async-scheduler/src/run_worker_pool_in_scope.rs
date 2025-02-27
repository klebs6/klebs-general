// ---------------- [ File: hydro2-async-scheduler/src/run_worker_pool_in_scope.rs ]
crate::ix!();

/// Creates a worker pool with the specified capacity and runs scheduling
/// inside a scoped thread. Returns an error if scheduling fails.
pub(crate) fn run_worker_pool_in_scope<T, F>(
    num_threads: usize,
    network: &Arc<AsyncMutex<Network<T>>>,
    scheduling_fn: F,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync + 'static,
    F: FnOnce(&WorkerPool<T>) -> Result<(), NetworkError>,
{
    std::thread::scope(|scope| -> Result<(), NetworkError> {
        let worker_pool = WorkerPool::<T>::new_in_scope(scope, num_threads, 256);
        scheduling_fn(&worker_pool)?;
        Ok(())
    })
}

#[cfg(test)]
mod run_worker_pool_in_scope_tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// 1) A minimal test ensuring that if the scheduling function
    ///    returns `Ok(())`, we get `Ok(())` out of `run_worker_pool_in_scope`.
    #[test]
    fn test_run_worker_pool_in_scope_success() {
        // Build some mock network if needed
        let network = Arc::new(AsyncMutex::new(build_mock_network()));

        // We'll define a scheduling_fn that just returns Ok.
        fn scheduling_fn_ok(_pool: &WorkerPool<u32>) -> Result<(), NetworkError> {
            eprintln!("scheduling_fn_ok => Doing nothing => returning Ok");
            Ok(())
        }

        let result = run_worker_pool_in_scope::<u32, _>(
            2, // num_threads
            &network,
            scheduling_fn_ok,
        );
        assert!(result.is_ok(), "Expected success when scheduling_fn returns Ok");
    }

    /// 2) Scheduling function returns an error => we expect `run_worker_pool_in_scope`
    ///    to propagate that error.
    #[test]
    fn test_run_worker_pool_in_scope_scheduling_fails() {
        let network = Arc::new(AsyncMutex::new(build_mock_network()));

        fn scheduling_fn_err(_pool: &WorkerPool<u32>) -> Result<(), NetworkError> {
            eprintln!("scheduling_fn_err => returning Error");
            Err(NetworkError::InvalidNode { node_idx: 99 })
        }

        let result = run_worker_pool_in_scope::<u32, _>(
            2,
            &network,
            scheduling_fn_err,
        );
        assert!(result.is_err(), "Expected an error to be returned");

        // Optionally match the exact error variant
        match result {
            Err(NetworkError::InvalidNode { node_idx }) => assert_eq!(node_idx, 99),
            _ => panic!("Expected NetworkError::InvalidNode(99)"),
        }
    }

    /// 3) If `num_threads = 0`, does your code handle it gracefully?
    ///    The function calls `WorkerPool::new_in_scope(scope, 0, 256)`.
    ///    Whether that is valid or not depends on your WorkerPool logic.
    ///    We'll assume it's valid but spawns only an aggregator thread or none at all.
    #[test]
    fn test_run_worker_pool_in_scope_zero_threads() {
        let network = Arc::new(AsyncMutex::new(build_mock_network()));

        fn scheduling_fn_zero(_pool: &WorkerPool<u32>) -> Result<(), NetworkError> {
            eprintln!("scheduling_fn_zero => no workers => returning Ok");
            Ok(())
        }

        let result = run_worker_pool_in_scope::<u32, _>(
            0, // zero threads
            &network,
            scheduling_fn_zero,
        );
        // Depending on your code, this might be Ok or an error. Check your design.
        assert!(result.is_ok(), "We expect 0 threads scenario to be handled or at least not panic");
    }

    /// 4) If `scheduling_fn` panics, ensure that we do not swallow the panic,
    ///    or see how it is handled. In normal usage, `std::thread::scope` will
    ///    unwind to the caller if a panic occurs inside the closure.
    #[test]
    #[should_panic(expected = "scheduled panic")]
    fn test_run_worker_pool_in_scope_panics() {
        let network = Arc::new(AsyncMutex::new(build_mock_network()));

        fn scheduling_fn_panics(_pool: &WorkerPool<u32>) -> Result<(), NetworkError> {
            panic!("scheduled panic");
        }

        // The test runner should see the panic from inside the scope
        let _ = run_worker_pool_in_scope::<u32, _>(
            2,
            &network,
            scheduling_fn_panics,
        );
    }

    /// 5) (Optional) Large concurrency test to ensure it doesn't crash or behave oddly.
    ///    We won't do real concurrency here, but you could do so in a more advanced test.
    #[test]
    fn test_run_worker_pool_in_scope_large_concurrency() {
        let network = Arc::new(AsyncMutex::new(build_mock_network()));

        fn scheduling_fn_large(_pool: &WorkerPool<u32>) -> Result<(), NetworkError> {
            eprintln!("scheduling_fn_large => pretend to schedule tasks => Ok");
            Ok(())
        }

        // Let's pick an arbitrarily large thread count
        let result = run_worker_pool_in_scope::<u32, _>(
            32, // 32 threads
            &network,
            scheduling_fn_large,
        );
        assert!(result.is_ok(), "Large concurrency should not fail by default");
    }

    // Helper to build a minimal mock network
    fn build_mock_network() -> Network<u32> {
        let mut net = Network::default();
        // Possibly add some nodes, edges, etc.
        net
    }
}
