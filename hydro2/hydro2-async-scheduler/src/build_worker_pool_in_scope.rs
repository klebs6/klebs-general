// ---------------- [ File: src/build_worker_pool_in_scope.rs ]
crate::ix!();

//========================================================
// Sub-subroutine #1: build a worker pool in scope
//========================================================
pub(crate) fn build_worker_pool_in_scope<'scope, T>(
    scope: &'scope Scope<'scope, '_>,
    scheduler: &AsyncScheduler,
    concurrency_limit: Arc<Semaphore>,
) -> Result<WorkerPool<'scope, T>, NetworkError>
where
    T: std::fmt::Debug + Send + Sync + 'scope,
{
    let num_threads = *scheduler.config().max_parallelism();
    eprintln!("build_worker_pool_in_scope => spawning {} threads", num_threads);

    let worker_pool = WorkerPool::<T>::new_in_scope(scope, num_threads, 256);
    // if worker_pool creation can fail, wrap in a result => done
    Ok(worker_pool)
}

#[cfg(test)]
mod build_worker_pool_in_scope_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_build_worker_pool_in_scope_basic() {

        let config = AsyncSchedulerConfigBuilder::default()
            .batching_strategy(BatchingStrategy::Immediate)
            .max_parallelism(2_usize)
            .enable_streaming(false)
            .checkpoint_callback(None)
            .build()
            .unwrap();

        let scheduler = AsyncScheduler::with_config(config);

        let concurrency = Arc::new(Semaphore::new(2));

        thread::scope(|scope| {
            let pool_res = build_worker_pool_in_scope::<usize>(
                scope, &scheduler, concurrency.clone()
            );
            assert!(pool_res.is_ok());
        });
    }

    // Add more negative/edge tests as needed (e.g., concurrency=0).
}
