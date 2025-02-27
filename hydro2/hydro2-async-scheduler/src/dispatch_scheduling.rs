// ---------------- [ File: src/dispatch_scheduling.rs ]
crate::ix!();

/// Dispatches the scheduling work depending on the batching strategy.
pub(crate) async fn dispatch_scheduling<'threads,T>(
    scheduler:         &AsyncScheduler,
    network:           Arc<AsyncMutex<Network<T>>>,
    concurrency_limit: Arc<Semaphore>,
    worker_pool:       &WorkerPool<'threads,T>,
    ready_nodes_rx:    tokio::sync::mpsc::Receiver<usize>,
    child_nodes_rx:    tokio::sync::mpsc::Receiver<usize>,
    completed_nodes:   SharedCompletedNodes,
    shared_in_degs:    Arc<AsyncMutex<Vec<usize>>>,
    node_count:        usize,
    stream_out_tx:     Option<StreamingOutputSender<T>>,
    mut perf:          &mut PerformanceStats,
    checkpoint_cb:     Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:    tokio::sync::mpsc::Sender<usize>,
    ready_nodes_tx:    tokio::sync::mpsc::Sender<usize>,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync,
{
    match scheduler.config().batching_strategy() {

        BatchingStrategy::Immediate => {
            eprintln!("execute_network => immediate scheduling");
            process_immediate(
                network,
                concurrency_limit,
                worker_pool,
                ready_nodes_rx,
                child_nodes_rx,
                completed_nodes,
                shared_in_degs,
                node_count,
                stream_out_tx,
                checkpoint_cb,
                child_nodes_tx,
                ready_nodes_tx,
            )
            .await?;
        }

        BatchingStrategy::Wave => {
            eprintln!("execute_network => wave scheduling");
            process_waves(
                network,
                concurrency_limit,
                worker_pool,
                ready_nodes_rx,
                child_nodes_rx,
                completed_nodes,
                shared_in_degs,
                stream_out_tx,
                checkpoint_cb,
                &mut perf,
                None,
                node_count,
                child_nodes_tx,
                ready_nodes_tx,
            )
            .await?;
        }

        BatchingStrategy::Threshold { chunk_size } => {
            eprintln!("execute_network => threshold({}) scheduling", chunk_size);
            process_waves(
                network,
                concurrency_limit,
                worker_pool,
                ready_nodes_rx,
                child_nodes_rx,
                completed_nodes,
                shared_in_degs,
                stream_out_tx,
                checkpoint_cb,
                &mut perf,
                Some(*chunk_size),
                node_count,
                child_nodes_tx,
                ready_nodes_tx,
            )
            .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod dispatch_scheduling_tests {
    use super::*;

    // A convenience helper for building an AsyncScheduler in tests
    fn mock_scheduler_with_batching_strategy(strategy: BatchingStrategy) -> AsyncScheduler {
        AsyncScheduler::new_test(strategy)
            .expect("Unable to create AsyncScheduler")
    }

    /// (A) No nodes => we set `node_count=0` => `process_immediate` sees done_count=0 == total=0 => immediate exit
    #[traced_test]
    fn test_dispatch_scheduling_immediate_no_nodes() {
        // Because we want aggregator + workers in real OS threads, we wrap the entire setup
        // in `std::thread::scope`. Then we can build a `WorkerPool::new_in_scope(...)`.
        thread::scope(|scope| {
            // 1) Build the scheduler
            let scheduler = mock_scheduler_with_batching_strategy(BatchingStrategy::Immediate);

            // 2) The concurrency limit
            let concurrency_limit = Arc::new(Semaphore::new(4));

            // 3) A real worker pool with aggregator + workers
            let worker_pool = WorkerPool::new_in_scope(scope, /*num_workers=*/2, /*buffer_size=*/16);

            // 4) Build minimal network => node_count=0 => means no real nodes
            let network = Arc::new(AsyncMutex::new(Network::<usize>::default()));
            let completed_nodes = SharedCompletedNodes::new();
            let shared_in_degs  = Arc::new(AsyncMutex::new(vec![]));
            let node_count      = 0; // so process_immediate can exit immediately

            // Channels for ready & child
            let (ready_tx, ready_rx) = mpsc::channel::<usize>(16);
            let (child_tx, child_rx) = mpsc::channel::<usize>(16);

            let mut perf      = PerformanceStats::start();
            let checkpoint_cb = None; // or Some(Arc::new(NoOpCheckpointCallback))

            // Clone them for usage inside the async block
            let ready_tx_for_async = ready_tx.clone();
            let child_tx_for_async = child_tx.clone();

            // 5) Because `dispatch_scheduling` is async, we block_on it in a small local runtime:
            //    But you can also do `futures::executor::block_on(...)` if you prefer.
            let result = {
                use tokio::runtime::Runtime;
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    dispatch_scheduling(
                        &scheduler,
                        network,
                        concurrency_limit,
                        &worker_pool,
                        ready_rx,
                        child_rx,
                        completed_nodes,
                        shared_in_degs,
                        node_count,
                        None,  // stream_out_tx
                        &mut perf,
                        checkpoint_cb,
                        child_tx_for_async,
                        ready_tx_for_async,
                    )
                    .await
                })
            };

            assert!(result.is_ok(), "Should exit cleanly with zero nodes");

            // 6) After returning => aggregator is likely idle. We can forcibly close channels:
            drop(ready_tx);
            drop(child_tx);

            // aggregator sees None => eventually finishes => the worker threads will exit
            // after receiving channel closure.

            // (end of the scope => aggregator + workers join)
        });
    }

    /// (B) With nodes => we enqueue some node indices => aggregator processes them => `process_immediate` finishes
    #[traced_test]
    fn test_dispatch_scheduling_immediate_with_nodes() {
        thread::scope(|scope| {
            let scheduler = mock_scheduler_with_batching_strategy(BatchingStrategy::Immediate);

            let concurrency_limit = Arc::new(Semaphore::new(4));

            // aggregator + 2 workers
            let worker_pool = WorkerPool::new_in_scope(scope, 2, 16);

            // A small network => e.g. 3 nodes in some trivial DAG
            let network         = triple_noop_operator_usize_network();
            let completed_nodes = SharedCompletedNodes::new();
            let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0,0,0]));
            let node_count      = 3;

            let (ready_tx, ready_rx) = mpsc::channel::<usize>(16);
            let (child_tx, child_rx) = mpsc::channel::<usize>(16);

            // We'll enqueue a few node indices => "ready" 0,1,2
            ready_tx.blocking_send(0).unwrap();
            ready_tx.blocking_send(1).unwrap();
            ready_tx.blocking_send(2).unwrap();

            let mut perf = PerformanceStats::start();
            let checkpoint_cb = None;

            // Clone them for usage inside the async block
            let ready_tx_for_async = ready_tx.clone();
            let child_tx_for_async = child_tx.clone();

            // Again we create a short-lived tokio runtime, or use any approach you prefer:
            let result = {
                use tokio::runtime::Runtime;
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    dispatch_scheduling(
                        &scheduler,
                        network,
                        concurrency_limit,
                        &worker_pool,
                        ready_rx,
                        child_rx,
                        completed_nodes,
                        shared_in_degs,
                        node_count,
                        None,
                        &mut perf,
                        checkpoint_cb,
                        child_tx_for_async,
                        ready_tx_for_async,
                    )
                    .await
                })
            };

            assert!(result.is_ok(), "Should process the 3 nodes, then exit");

            // forcibly close the channels
            drop(ready_tx);
            drop(child_tx);

            // aggregator sees None => aggregator + workers exit
        });
    }
}
