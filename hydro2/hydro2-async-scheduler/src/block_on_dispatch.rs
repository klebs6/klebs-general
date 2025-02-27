// ---------------- [ File: src/block_on_dispatch.rs ]
crate::ix!();

pub(crate) fn block_on_dispatch<'threads, T>(
    scheduler:           &AsyncScheduler,
    network:             Arc<AsyncMutex<Network<T>>>,
    concurrency_limit:   Arc<Semaphore>,
    worker_pool:         &WorkerPool<'threads, T>,
    ready_nodes_rx:      mpsc::Receiver<usize>,
    child_nodes_rx:      mpsc::Receiver<usize>,
    completed_nodes:     SharedCompletedNodes,
    shared_in_degs:      Arc<AsyncMutex<Vec<usize>>>,
    node_count:          usize,
    stream_out_tx:       Option<StreamingOutputSender<T>>,
    perf:                &mut PerformanceStats,
    checkpoint_cb:       Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:      mpsc::Sender<usize>,
    ready_nodes_tx:      mpsc::Sender<usize>,
) -> Result<(), NetworkError>
where
    T: std::fmt::Debug + Send + Sync + 'threads
{
    eprintln!("blocking on dispatch");

    futures::executor::block_on(dispatch_scheduling(
        scheduler,
        network,
        concurrency_limit,
        worker_pool,
        ready_nodes_rx,
        child_nodes_rx,
        completed_nodes,
        shared_in_degs,
        node_count,
        stream_out_tx,
        perf,
        checkpoint_cb,
        child_nodes_tx,
        ready_nodes_tx,
    ))
}

#[cfg(test)]
mod block_on_dispatch_tests {
    use super::*;

    #[traced_test]
    async fn test_block_on_dispatch_no_nodes() -> Result<(), NetworkError> {

        // 1) Build scheduler config with immediate scheduling
        let config = AsyncSchedulerConfigBuilder::default()
            .batching_strategy(BatchingStrategy::Immediate)
            .max_parallelism(2_usize)
            .enable_streaming(false)
            .checkpoint_callback(None)
            .build().unwrap();

        let scheduler = AsyncScheduler::with_config(config);

        // 2) Minimal network & concurrency
        let net         = empty_usize_network();
        let concurrency = Arc::new(Semaphore::new(2));

        // 3) Additional structures for scheduling
        let completed_nodes = SharedCompletedNodes::new();
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0,0,0])); 
        let node_count      = 0;
        let tx_out: Option<StreamingOutputSender<usize>> = None;
        let mut perf        = PerformanceStats::start();
        let cb: Option<Arc<dyn CheckpointCallback>> = None;

        // We do all aggregator + worker creation + usage in a synchronous scope:
        std::thread::scope(|scope| {
            // (A) Build the actual worker pool in-scope
            let worker_pool = WorkerPool::new_in_scope(scope, /*num_workers=*/2, /*buffer_size=*/10);

            // (B) Our channels for ready-nodes and child-nodes
            let (ready_tx,  ready_rx)  = mpsc::channel(10);
            let (child_tx,  child_rx)  = mpsc::channel(10);

            // 4) Now run the code under test with zero enqueued nodes
            //    We do NOT drop(ready_tx) yet—if we do, we can’t pass it inside `block_on_dispatch`.
            let res = block_on_dispatch(
                &scheduler, 
                net.clone(), 
                concurrency.clone(), 
                &worker_pool, 
                ready_rx,     // sees None as soon as original is dropped
                child_rx,     // same
                completed_nodes.clone(), 
                shared_in_degs.clone(), 
                node_count, 
                tx_out.clone(),
                &mut perf, 
                cb.clone(),
                child_tx.clone(),
                ready_tx.clone(),
            );

            assert!(res.is_ok(), "Should exit cleanly with no nodes enqueued");

            // If we want aggregator to see `None` channels now, we can do it here:
            drop(ready_tx);
            drop(child_tx);

            // aggregator + workers see None => they eventually finish
        });  // aggregator + worker threads join here

        Ok(())
    }

    #[traced_test]
    async fn test_block_on_dispatch_with_nodes() -> Result<(),NetworkError> {

        let test_result = tokio::time::timeout(Duration::from_secs(10), async {
            eprintln!("starting test");
            let config = AsyncSchedulerConfigBuilder::default()
                .batching_strategy(BatchingStrategy::Immediate)
                .max_parallelism(2_usize)
                .enable_streaming(false)
                .checkpoint_callback(None)
                .build().unwrap();
            let scheduler = AsyncScheduler::with_config(config);

            let net         = triple_noop_operator_usize_network();
            let concurrency = Arc::new(Semaphore::new(2));

            let completed_nodes = SharedCompletedNodes::new();
            let shared_in_degs  = Arc::new(AsyncMutex::new(vec![0,0,0]));
            let node_count      = 3;
            let tx_out: Option<StreamingOutputSender<TestWireIO<i32>>> = None;
            let mut perf        = PerformanceStats::start();
            let cb: Option<Arc<dyn CheckpointCallback>> = None;

            std::thread::scope(|scope| {
                // aggregator + 2 workers
                let worker_pool = WorkerPool::new_in_scope(scope, 2, 10);

                // Create channels
                let (ready_tx, ready_rx) = mpsc::channel(10);
                let (child_tx, child_rx) = mpsc::channel(10);

                // Enqueue some nodes => "ready" nodes
                ready_tx.try_send(0).unwrap();
                ready_tx.try_send(1).unwrap();
                ready_tx.try_send(2).unwrap();

                // Now run block_on_dispatch
                // We'll NOT drop them until after block_on, so aggregator can read them.
                let res = block_on_dispatch(
                    &scheduler,
                    net.clone(),
                    concurrency.clone(),
                    &worker_pool,
                    ready_rx,
                    child_rx,
                    completed_nodes.clone(),
                    shared_in_degs.clone(),
                    node_count,
                    tx_out.clone(),
                    &mut perf,
                    cb.clone(),
                    child_tx.clone(),
                    ready_tx.clone(),
                );
                assert!(res.is_ok(), "Should process the enqueued nodes, then exit");

                // Now forcibly close:
                drop(ready_tx);
                drop(child_tx);

                // aggregator sees None => aggregator + workers eventually exit
            }); // aggregator + worker threads join

            Ok(())
        })
        .await;

        match test_result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => panic!("test_block_on_dispatch timed out after 10 seconds"),
        }
    }
}
