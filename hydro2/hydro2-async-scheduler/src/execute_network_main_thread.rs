// ---------------- [ File: src/execute_network_main_thread.rs ]
crate::ix!();

/// Runs the main scheduling logic in a scoped thread.
///  1) Builds a WorkerPool (aggregator + N workers).
///  2) Builds in-degrees.
///  3) Initializes zero-degree nodes.
///  4) Dispatches scheduling (immediate, wave, or threshold).
///  5) Finally, drops channels so that the scheduling loop can exit.
///
/// After returning, the aggregator + workers are either done or will finish
/// soon. This function itself is synchronous (non-async) because we call
/// `block_on_*` in several places.
pub(crate) fn execute_network_main_thread<'scope, T>(
    scope:              &'scope std::thread::Scope<'scope, '_>,
    scheduler:          &AsyncScheduler,
    network:            Arc<AsyncMutex<Network<T>>>,
    concurrency_limit:  Arc<tokio::sync::Semaphore>,
    ready_nodes_tx:     tokio::sync::mpsc::Sender<usize>,
    ready_nodes_rx:     tokio::sync::mpsc::Receiver<usize>,
    child_nodes_tx:     tokio::sync::mpsc::Sender<usize>,
    child_nodes_rx:     tokio::sync::mpsc::Receiver<usize>,
    stream_out_tx:      Option<StreamingOutputSender<T>>,
    checkpoint_cb:      Option<Arc<dyn CheckpointCallback>>,
    perf:               &mut PerformanceStats,
) -> Result<(), NetworkError>
where
    T: std::fmt::Debug + Send + Sync + 'scope,
{
    // (A) Build worker pool in a scoped thread
    let worker_pool = build_worker_pool_in_scope(scope, scheduler, concurrency_limit.clone())?;

    // (B) Gather node count + edges
    let (node_count, edges) = gather_node_count_and_edges(&network)?;

    // (C) Build in-degrees (blocking call)
    let shared_in_degs = block_on_build_in_degrees(&edges, node_count)?;

    // (D) Completed
    let completed_nodes = SharedCompletedNodes::new();

    // (E) Initialize zero-degree nodes
    let zero_count = block_on_init_zero_degree(
        &shared_in_degs,
        node_count,
        &ready_nodes_tx,
        &child_nodes_tx,
        &worker_pool,
    )?;
    eprintln!("initialize_zero_degree_nodes => enqueued={}", zero_count);

    // (F) If single node, forcibly close aggregator + Freed channel
    if node_count == 1 {
        eprintln!("single-node => forcibly close Freed + aggregator => short-circuit return");
        force_close_for_single_node(ready_nodes_tx, child_nodes_tx, &worker_pool);

        perf.end();
        eprintln!("single-node => scheduling complete => perf={:?}", perf);
        return Ok(());
    }

    // (G) Dispatch scheduling (Immediate, Wave, or Threshold)
    //     We block on the async scheduling function:
    let result = block_on_dispatch(
        scheduler,
        network,
        concurrency_limit,
        &worker_pool,
        ready_nodes_rx,
        child_nodes_rx,
        completed_nodes.clone(),
        Arc::clone(&shared_in_degs),
        node_count,
        stream_out_tx,
        perf,
        checkpoint_cb,
        child_nodes_tx.clone(),  // pass a clone to the scheduling routine
        ready_nodes_tx.clone(),  // pass a clone to the scheduling routine
    )?;

    // -----------------------------------------------------------
    // (H) After scheduling finishes, ensure everything can exit:
    //     1) Drop ready_nodes_tx so `ready_nodes_rx` yields None.
    //     2) Drop child_nodes_tx so `child_nodes_rx` yields None.
    //     3) Force aggregator to see "None" by dropping main tasks channel.
    // -----------------------------------------------------------
    eprintln!("execute_network_main_thread => forcibly dropping Freed child_nodes_tx + ready_nodes_tx");
    drop(child_nodes_tx);
    drop(ready_nodes_tx);

    // aggregator channel closure
    worker_pool.close_main_tasks_channel();

    // finalize perf
    perf.end();
    eprintln!("execute_network_main_thread => done => perf={:?}", perf);

    Ok(result)
}

#[cfg(test)]
mod execute_network_main_thread_tests {
    use super::*;
    use std::thread;

    #[traced_test]
    async fn test_execute_network_main_thread_ok() {
        // Minimal demonstration: a network with node_count > 1 => normal scheduling, etc.
        // Because this still depends on WorkerPool, we might need a "mock" or "dummy" approach.
        // For brevity, we skip full mocking. Just ensure it compiles & doesn't panic.

        let network = Arc::new(AsyncMutex::new(mock_network_with_n_nodes(4)));

        let config = AsyncSchedulerConfigBuilder::default()
            .batching_strategy(BatchingStrategy::Immediate)
            .max_parallelism(2_usize)
            .enable_streaming(false)
            .checkpoint_callback(None)
            .build()
            .unwrap();

        let scheduler = AsyncScheduler::with_config(config);

        let concurrency_limit    = Arc::new(Semaphore::new(2));
        let (ready_tx, ready_rx) = mpsc::channel(10);
        let (child_tx, child_rx) = mpsc::channel(10);
        let stream_out_tx        = None;
        let checkpoint_cb        = None;
        let mut perf             = PerformanceStats::start();

        thread::scope(|scope| {
            let result = execute_network_main_thread(
                scope,
                &scheduler,
                network.clone(),
                concurrency_limit.clone(),
                ready_tx,
                ready_rx,
                child_tx,
                child_rx,
                stream_out_tx,
                checkpoint_cb,
                &mut perf,
            );
            assert!(result.is_ok(), "Should succeed for a multi-node scenario");
        });
    }

    #[traced_test]
    async fn test_execute_network_main_thread_single_node() {

        // If node_count=1, we do the single-node fast path => forcibly close Freed + aggregator
        let network = Arc::new(AsyncMutex::new(mock_network_with_n_nodes(1)));

        let scheduler 
            = AsyncSchedulerBuilder::default()
            .config(AsyncSchedulerConfigBuilder::default()
                .batching_strategy(BatchingStrategy::Immediate)
                .max_parallelism(1_usize)
                .enable_streaming(false)
                .checkpoint_callback(None)
                .build()
                .unwrap())
            .build()
            .unwrap();

        let concurrency_limit    = Arc::new(Semaphore::new(1));
        let (ready_tx, ready_rx) = mpsc::channel(10);
        let (child_tx, child_rx) = mpsc::channel(10);
        let mut perf             = PerformanceStats::start();

        std::thread::scope(|scope| {
            let res = execute_network_main_thread(
                scope,
                &scheduler,
                network.clone(),
                concurrency_limit.clone(),
                ready_tx,
                ready_rx,
                child_tx,
                child_rx,
                None, None,
                &mut perf,
            );
            assert!(res.is_ok());
        });
    }

    // A helper to build a mock network with n nodes and 0 edges for demonstration
    fn mock_network_with_n_nodes(n: usize) -> Network<TestWireIO<i32>> {
        // Typically you'd have a real Network structure.
        let mut net = Network::default();
        for idx in 0..n {
            let op = NoOpOperator::with_name(format!("no_op operator {}", idx));
            net.nodes_mut().push(node![idx => op]);
        }
        // no edges => everything might be zero in-degree
        net
    }
}
