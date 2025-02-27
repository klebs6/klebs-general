// ---------------- [ File: hydro2-async-scheduler/src/execute_network.rs ]
crate::ix!();

impl AsyncScheduler {

    pub fn execute_network<'threads, T>(
        &self,
        network: Arc<AsyncMutex<Network<T>>>,
    ) -> Result<(PerformanceStats, Option<StreamingOutput<T>>), NetworkError>
    where
        T: std::fmt::Debug + Send + Sync + 'threads
    {
        eprintln!(
            "execute_network: Starting. Strategy={:?}, concurrency={}",
            self.config().batching_strategy(),
            self.config().max_parallelism()
        );

        // 1) Validate outside the scope
        futures::executor::block_on(validate_network(&network))?;

        // 2) concurrency limit
        let concurrency_limit = Arc::new(Semaphore::new(*self.config().max_parallelism()));

        // 3) Freed child channels
        let (ready_nodes_tx, ready_nodes_rx) = mpsc::channel::<usize>(256);
        let (child_nodes_tx, child_nodes_rx) = mpsc::channel::<usize>(256);

        // 4) optional streaming
        let (stream_out_tx, stream_out_rx) =
            if *self.config().enable_streaming() {
                eprintln!("execute_network => streaming enabled");
                let (tx, rx) = mpsc::channel::<(usize, NetworkNodeIoChannelArray<T>)>(256);
                (Some(tx), Some(rx))
            } else {
                (None, None)
            };

        let checkpoint_cb = self.config().checkpoint_callback().clone();

        // prepare stats + final stream
        let mut perf = PerformanceStats::start();
        let final_stream = stream_out_rx;
        let num_threads = *self.config().max_parallelism();

        eprintln!("execute_network => building worker pool with {} threads (scoped)", num_threads);

        // 5) run in a scoped thread => we refactor that logic
        std::thread::scope(|scope| -> Result<(), NetworkError> {
            execute_network_main_thread(
                scope,
                &self,
                network,
                concurrency_limit,
                ready_nodes_tx,
                ready_nodes_rx,
                child_nodes_tx,
                child_nodes_rx,
                stream_out_tx,
                checkpoint_cb,
                &mut perf,
            )
        })?;

        eprintln!("execute_network => done => returning perf={:?}", perf);
        Ok((perf, final_stream))
    }
}

#[cfg(test)]
mod execute_network_tests {

    use super::*;

    async fn test_fn() -> Result<(),NetworkError> {
        // Build a real or nearly-real network with known topological ordering
        let network = Arc::new(AsyncMutex::new(build_test_network()));

        let scheduler = AsyncScheduler::new_test(BatchingStrategy::Immediate)?;

        let result = scheduler.execute_network(network.clone());
        assert!(result.is_ok());

        let (perf, stream_out) = result.unwrap();
        // Check that performance stats are within expected range, etc.
        // Check streaming output correctness if streaming was enabled
        Ok(())
    }

    #[traced_test]
    fn test_execute_network_integration() -> Result<(),NetworkError> {
        let rt = TokioRuntime::new().unwrap();
        rt.block_on(async {
            test_fn().await
        });
        Ok(())
    }

    fn build_test_network() -> Network<u32> {
        // Build a small network with known topology
        let mut net = Network::default();
        // ...
        net
    }
}
