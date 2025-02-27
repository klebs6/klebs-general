// ---------------- [ File: hydro2-async-scheduler/src/poll_worker_results.rs ]
crate::ix!();

pub async fn poll_worker_results<T>(
    worker_pool:     &WorkerPool<'_, T>,
    completed_nodes: &SharedCompletedNodes,
    in_flight:       &mut InFlightCounter
) -> Result<(),NetworkError>
where
    T: std::fmt::Debug + Send + Sync,
{
    eprintln!("poll_worker_results => enter => checking for TaskResult items");

    while let Some(task_res) = worker_pool.try_recv_result().await {
        let node_idx = *task_res.node_idx();
        eprintln!("poll_worker_results => got TaskResult => node_idx={}", node_idx);

        // Check if there's an error
        if let Some(err) = task_res.error() {
            eprintln!("poll_worker_results => error => returning Err={:?}", err);
            // in_flight.decrement() only if you truly had in_flight++ for that node,
            // but typically you do if a node was submitted => in_flight++.
            in_flight.decrement();

            // DO NOT insert node_idx into completed_nodes if it's an error
            return Err(err.clone());
        } else {
            // No error => mark node as completed
            completed_nodes
                .insert(node_idx)
                .await
                .expect(&format!("could not insert completed node {}", node_idx));

            // in_flight--
            in_flight.decrement();
            eprintln!("poll_worker_results => in_flight-- => now={}", in_flight.get());
        }
    }

    eprintln!("poll_worker_results => no more TaskResult => returning Ok");
    Ok(())
}

#[cfg(test)]
mod poll_worker_results_tests {
    use super::*;

    #[traced_test]
    async fn test_poll_worker_results_no_errors() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        // Suppose a worker pool that yields some successful tasks
        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(1_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(2_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
        ])?;

        // We must pass a `completed_nodes` argument to `poll_worker_results`.
        let completed_nodes = SharedCompletedNodes::new();

        // Now call the updated function with both arguments.
        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(res.is_ok());

        // Optionally check that `completed_nodes` has length=2 now:
        assert_eq!(completed_nodes.len().await, 2);

        Ok(())
    }

    #[traced_test]
    async fn test_poll_worker_results_with_error() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(3_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 3 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
        ])?;

        let completed_nodes = SharedCompletedNodes::new();

        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(res.is_err());
        // Possibly match the error variant to ensure it's the expected type
        Ok(())
    }

    /// 1) Ensures that if the channel is empty (no `TaskResult`),
    ///    we simply return `Ok` without error.
    #[traced_test]
    async fn test_poll_worker_results_empty() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![])?;
        let completed_nodes = SharedCompletedNodes::new();

        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(res.is_ok(), "polling an empty results channel should yield Ok");

        // Also confirm no completed nodes
        assert_eq!(completed_nodes.len().await, 0);

        Ok(())
    }

    /// 2) A mixture of successes and errors; we confirm that as soon
    ///    as we hit an error, the function returns and does NOT process
    ///    subsequent results.
    #[traced_test]
    async fn test_poll_worker_results_partial_error() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(10_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
            // The first error encountered
            TaskResultBuilder::default()
                .node_idx(11_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 11 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
            // This result is after the error, we want to ensure we never get here
            TaskResultBuilder::default()
                .node_idx(12_usize)
                .error(None)
                .freed_children(vec![])
                .build()
                .unwrap(),
        ])?;

        let completed_nodes = SharedCompletedNodes::new();

        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(
            res.is_err(),
            "We expect an error from node_idx=11, so poll_worker_results should return immediately"
        );

        // Confirm that we only see the success for node_idx=10 before the error
        assert_eq!(completed_nodes.len().await, 1, "Only node_idx=10 was processed successfully");

        Ok(())
    }

    /// 3) Multiple errors in the queue. The function should return on
    ///    the first error and never observe the second error.
    #[traced_test]
    async fn test_poll_worker_results_multiple_errors() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let (worker_pool, _rx) = mock_worker_pool_with_results(vec![
            TaskResultBuilder::default()
                .node_idx(20_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 20 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
            // Another error after the first
            TaskResultBuilder::default()
                .node_idx(21_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 21 }))
                .freed_children(vec![])
                .build()
                .unwrap(),
        ])?;

        let completed_nodes = SharedCompletedNodes::new();

        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        // The function returns as soon as it sees the error from node_idx=20
        // We do not see the second error from node_idx=21.
        assert!(res.is_err());

        // Confirm no successes prior to first error
        assert_eq!(completed_nodes.len().await, 0);

        Ok(())
    }

    /// 4) Many success results. This is just a slight expansion of
    ///    the `no_errors` scenario to confirm we can handle more items.
    #[traced_test]
    async fn test_poll_worker_results_many_successes() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let results: Vec<TaskResult> = (0_usize..5)
            .map(|i| {
                TaskResultBuilder::default()
                    .node_idx(i)
                    .error(None)
                    .freed_children(vec![])
                    .build()
                    .unwrap()
            })
            .collect();

        let (worker_pool, _rx) = mock_worker_pool_with_results(results)?;

        let completed_nodes = SharedCompletedNodes::new();

        let res = poll_worker_results(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(res.is_ok(), "All five tasks were successful => Ok");

        // Check that we have 5 completed nodes
        assert_eq!(completed_nodes.len().await, 5);

        Ok(())
    }
}
