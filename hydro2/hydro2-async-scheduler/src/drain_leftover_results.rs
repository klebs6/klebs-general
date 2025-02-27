// ---------------- [ File: hydro2-async-scheduler/src/drain_leftover_results.rs ]
crate::ix!();

/// Drains leftover results from the worker pool after the main loop finishes.
/// If any error is returned, we stop and return that error. Otherwise Ok.
pub(crate) async fn drain_leftover_results(
    worker_pool: &WorkerPool<'_, impl Debug + Send + Sync>,
) -> Result<(), NetworkError> {
    loop {
        if let Some(task_res) = worker_pool.try_recv_result().await {
            if let Some(err) = task_res.error() {
                eprintln!("process_immediate => leftover worker error={:?}", err);
                return Err(err.clone());
            }
            eprintln!(
                "process_immediate => leftover worker result node_idx={}",
                task_res.node_idx()
            );
        } else {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod drain_leftover_results_tests {
    use super::*;

    #[traced_test]
    async fn test_drain_leftover_results_ok() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(10_usize)
                .freed_children(vec![])
                .error(None)
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_ok());
        Ok(())
    }

    #[traced_test]
    async fn test_drain_leftover_results_with_error() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(11_usize)
                .freed_children(vec![])
                .error(Some(NetworkError::InvalidNode { node_idx: 11 }))
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_err());
        Ok(())
    }

    /// 1) If there are no leftover results, we simply return `Ok`.
    #[traced_test]
    async fn test_drain_leftover_results_empty() -> Result<(),NetworkError> {
        let (worker_pool,_rx) = mock_worker_pool_with_results(vec![])?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_ok(), "Empty leftover results should yield Ok");
        Ok(())
    }

    /// 2) Multiple successes => we finish reading them all and return `Ok`.
    #[traced_test]
    async fn test_drain_leftover_results_all_successes() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(0_usize)
                .freed_children(vec![])
                .error(None)
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(1_usize)
                .freed_children(vec![])
                .error(None)
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(2_usize)
                .freed_children(vec![])
                .error(None)
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_ok(), "All leftover results have no errors => Ok");
        Ok(())
    }

    /// 3) Immediately encounters an error on the first leftover result.
    ///    We should stop and return that error right away.
    #[traced_test]
    async fn test_drain_leftover_results_first_error() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(10_usize)
                .freed_children(vec![])
                .error(Some(NetworkError::InvalidNode { node_idx: 10 }))
                .build()
                .unwrap(),
            // even if we have more results, we won't see them
            TaskResultBuilder::default()
                .node_idx(11_usize)
                .freed_children(vec![])
                .error(None)
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_err(), "Should fail on the first leftover error (node_idx=10)");
        Ok(())
    }

    /// 4) Error in the middle: some successes first, then an error, then more results.
    ///    The function must return on the error and never process subsequent items.
    #[traced_test]
    async fn test_drain_leftover_results_error_in_middle() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(0_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(1_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
            // Error here:
            TaskResultBuilder::default()
                .node_idx(2_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 2 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
            // Another success afterwards, which we should never reach:
            TaskResultBuilder::default()
                .node_idx(3_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_err(), "Should stop upon encountering the error at node_idx=2");
        Ok(())
    }

    /// 5) Multiple errors: the function stops at the first error
    ///    and never processes the second error or subsequent items.
    #[traced_test]
    async fn test_drain_leftover_results_multiple_errors() -> Result<(),NetworkError> {
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(5_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 5 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(6_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 6 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
        ];
        let (worker_pool,_rx) = mock_worker_pool_with_results(results)?;
        let res = drain_leftover_results(&worker_pool).await;
        assert!(res.is_err(), "Stopped on the first error at node_idx=5");
        // The second error at node_idx=6 is never even seen.
        Ok(())
    }
}
