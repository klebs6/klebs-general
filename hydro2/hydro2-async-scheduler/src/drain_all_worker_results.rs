// ---------------- [ File: src/drain_all_worker_results.rs ]
crate::ix!();

pub(crate) async fn drain_all_worker_results_idle_based<T>(
    worker_pool:     &WorkerPool<'_, T>,
    completed_nodes: &SharedCompletedNodes,
    in_flight:       &mut InFlightCounter,
) -> Result<(), NetworkError> 
where T: Debug + Send + Sync,
{
    let overall_deadline = Instant::now() + Duration::from_secs(1);
    let mut last_new_completion = Instant::now();

    loop {
        let old_count = { completed_nodes.len().await };

        // poll a single time
        poll_worker_results(worker_pool, completed_nodes,in_flight).await?;

        let new_count = { completed_nodes.len().await };
        if new_count > old_count {
            // we got new completions => reset idle timer
            last_new_completion = Instant::now();
        } else {
            // no new completions => check idle
            if Instant::now() - last_new_completion > Duration::from_millis(200) {
                // 200ms with no new completions => done
                break;
            }
        }

        // optional overall limit
        if Instant::now() >= overall_deadline {
            break; // or return Ok or Err, your design choice
        }
        // short sleep
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    Ok(())
}


#[cfg(test)]
mod drain_all_worker_results_tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::time::Duration;

    /// A helper that builds a WorkerPool and immediately pushes
    /// a list of `TaskResult`s so we can test how `drain_all_worker_results(...)`
    /// processes them.
    fn mock_pool_with_initial_results(
        results: Vec<TaskResult>
    ) -> Result<(Arc<WorkerPool<'static,u32>>, mpsc::Receiver<TaskItem<'static,u32>>), NetworkError>
    {
        // We rely on the “mock_worker_pool_with_results(...)” from your code
        // which returns (WorkerPool, main_tasks_rx).
        let (pool, rx) = mock_worker_pool_with_results(results)?;

        // Wrap the newly built pool in Arc so we can share it in tasks
        Ok((Arc::new(pool), rx))
    }

    #[traced_test]
    async fn test_drain_all_worker_results_empty_to_start() -> Result<(), NetworkError> {
        // WorkerPool with no initial results => poll => empty => done
        let (worker_pool, _rx) = mock_pool_with_initial_results(vec![])?;
        let completed_nodes = SharedCompletedNodes::new();

        let mut in_flight = InFlightCounter::default();

        let outcome = drain_all_worker_results_idle_based(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(outcome.is_ok());
        assert_eq!(completed_nodes.len().await, 0);
        Ok(())
    }

    #[traced_test]
    async fn test_drain_all_worker_results_single_pass_all_ok() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let results = vec![
            TaskResultBuilder::default().node_idx(10_usize).error(None).build().unwrap(),
            TaskResultBuilder::default().node_idx(11_usize).error(None).build().unwrap(),
            TaskResultBuilder::default().node_idx(12_usize).error(None).build().unwrap(),
        ];
        let (worker_pool, _rx) = mock_pool_with_initial_results(results)?;
        let completed_nodes = SharedCompletedNodes::new();

        // Instead of `usize::MAX`:
        let outcome = drain_all_worker_results_idle_based(&worker_pool, &completed_nodes, &mut in_flight).await;

        assert!(outcome.is_ok());
        assert_eq!(completed_nodes.len().await, 3);
        assert_eq!(completed_nodes.as_slice().await, &[10,11,12]);
        Ok(())
    }

    #[traced_test]
    async fn test_drain_all_worker_results_error_immediate() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        // The first poll sees an error => we return it
        let results = vec![
            TaskResultBuilder::default()
                .node_idx(20_usize)
                .error(Some(NetworkError::InvalidNode { node_idx: 20 }))
                .build()
                .unwrap(),
            // subsequent items won't be processed
            TaskResultBuilder::default()
                .node_idx(21_usize)
                .error(None)
                .build()
                .unwrap(),
        ];
        let (worker_pool, _rx) = mock_pool_with_initial_results(results)?;
        let completed_nodes = SharedCompletedNodes::new();

        let outcome = drain_all_worker_results_idle_based(&worker_pool, &completed_nodes, &mut in_flight).await;
        assert!(outcome.is_err(), "Should see the error from node_idx=20");

        assert_eq!(completed_nodes.len().await, 0, "No successes recorded before error");
        Ok(())
    }

    /// The second wave includes an error => we expect to see that error on the second pass
    #[traced_test]
    async fn test_drain_all_worker_results_error_on_second_pass() -> Result<(), NetworkError> {

        let mut in_flight = InFlightCounter::default();

        let (worker_pool, _rx) = WorkerPool::<usize>::new_test_dummy()?;
        let worker_pool = Arc::new(worker_pool);

        // push first wave => 2 successes
        worker_pool.push_results_into_worker_pool(vec![
            TaskResultBuilder::default()
                .node_idx(40_usize)
                .error(None)
                .build()
                .unwrap(),
            TaskResultBuilder::default()
                .node_idx(41_usize)
                .error(None)
                .build()
                .unwrap(),
        ]);

        let completed_nodes = SharedCompletedNodes::new();
        let worker_pool_clone = Arc::clone(&worker_pool);
        let completed_nodes_clone = completed_nodes.clone();

        // After a short delay => push an error for node_idx=42
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            worker_pool_clone.push_results_into_worker_pool(vec![
                TaskResultBuilder::default()
                    .node_idx(42_usize)
                    .error(Some(NetworkError::InvalidNode { node_idx: 42 }))
                    .build()
                    .unwrap()
            ]);
        });

        let outcome = drain_all_worker_results_idle_based(&worker_pool, &completed_nodes_clone, &mut in_flight).await;
        // We expect an error from node_idx=42
        assert!(outcome.is_err());

        // We do have 2 successes prior to that
        assert_eq!(completed_nodes.len().await, 2, "Should have recorded node_idx=40,41 before the error");
        let mut sorted = completed_nodes.clone();
        assert_eq!(sorted.as_slice().await, &[40,41]);
        Ok(())
    }

    #[traced_test]
    async fn test_drain_all_worker_results_multiple_passes_needed() -> Result<(), NetworkError> {

        let mut in_flight = InFlightCounter::default();

        // set up mock WorkerPool
        let (worker_pool, _rx) = WorkerPool::<u32>::new_test_dummy()?;
        let worker_pool = Arc::new(worker_pool);

        // push wave #1 => nodes=30,31
        worker_pool.push_results_into_worker_pool(vec![
            TaskResultBuilder::default()
            .node_idx(30_usize)
            .error(None)
            .build()
            .unwrap(),
            TaskResultBuilder::default()
            .node_idx(31_usize)
            .error(None)
            .build()
            .unwrap(),
        ]);

        // spawn a task that, after 50ms, pushes node=32
        let wp2 = Arc::clone(&worker_pool);
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            wp2.push_results_into_worker_pool(vec![
                TaskResultBuilder::default()
                .node_idx(32_usize)
                .error(None)
                .build()
                .unwrap(),
            ]);
        });

        // We expect 3 completions total
        let completed_nodes = SharedCompletedNodes::new();
        let outcome = drain_all_worker_results_idle_based(&worker_pool, &completed_nodes, &mut in_flight).await;

        assert!(outcome.is_ok(), "Should eventually get node=32 as well");
        assert_eq!(completed_nodes.len().await, 3, "We wanted 3 completions: 30,31,32");
        Ok(())
    }
}
