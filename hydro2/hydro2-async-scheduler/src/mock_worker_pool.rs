// ---------------- [ File: hydro2-async-scheduler/src/mock_worker_pool.rs ]
crate::ix!();

//===========================================================
// Helper: Build a WorkerPool that yields the given results
//===========================================================
#[cfg(test)]
pub fn mock_worker_pool_with_results(
    results: Vec<TaskResult>
) -> Result<(WorkerPool<'static, u32>, mpsc::Receiver<crate::TaskItem<'static, u32>>), NetworkError>
{
    let (pool, rx) = WorkerPool::new_test_dummy()?;
    pool.push_results_into_worker_pool(results);
    Ok((pool, rx))
}

impl<'threads, T> WorkerPool<'threads, T>
where
    T: Default + Debug + Send + Sync + 'threads,
{
    /// Returns a WorkerPool plus the main_tasks_rx so the test can keep
    /// it alive, preventing channel closure.
    #[cfg(test)]
    pub fn new_test_dummy() -> Result<(Self, tokio::sync::mpsc::Receiver<TaskItem<'threads, T>>), NetworkError> {
        let (main_tasks_tx, main_tasks_rx) = tokio::sync::mpsc::channel::<TaskItem<'threads, T>>(16);
        let (results_tx, results_rx)      = tokio::sync::mpsc::channel::<TaskResult>(16);

        let dummy_threads = Vec::new();

        let pool = WorkerPoolBuilder::default()
            .main_tasks_tx(main_tasks_tx)
            .threads(dummy_threads)
            .results_rx(AsyncMutex::new(results_rx))
            .results_tx_for_test(Some(results_tx))   // <--- KEY
            .build()
            .unwrap();

        Ok((pool, main_tasks_rx))
    }

        /// A "test dummy" worker pool that guarantees `submit(...)` fails
    /// because the main tasks channel is closed from the start.
    ///
    /// We return `(WorkerPool, Receiver<TaskItem>)` so you can keep the same
    /// function signature, but the returned `Receiver` is just a fake placeholder.
    /// The *real* channel was dropped, ensuring an error from `submit(...)`.
    #[cfg(test)]
    pub fn new_test_dummy_causing_error() -> Result<(Self, tokio::sync::mpsc::Receiver<TaskItem<'threads, T>>), NetworkError> 
    {
        use tokio::sync::mpsc;

        // Create the real channel with capacity=1
        let (main_tasks_tx, real_main_tasks_rx) = mpsc::channel::<TaskItem<'threads, T>>(1);

        // Also create a results channel
        let (results_tx, results_rx) = mpsc::channel::<TaskResult>(1);

        // Because we drop the real receiver immediately, the channel is closed.
        drop(real_main_tasks_rx);

        // So any call to `submit(...)` -> channel.send(...) => error
        let dummy_threads = Vec::new();

        let pool = WorkerPoolBuilder::default()
            .main_tasks_tx(main_tasks_tx)
            .threads(dummy_threads)
            .results_rx(AsyncMutex::new(results_rx))
            .results_tx_for_test(Some(results_tx))   // <--- KEY
            .build()
            .unwrap();

        // Return a "fake" receiver that isn't actually used
        // but matches the function signature you want.
        let (fake_tx, fake_rx) = mpsc::channel::<TaskItem<'threads, T>>(1);

        Ok((pool, fake_rx))
    }

    /// A test-only constructor that forces `.submit(...)` to return
    /// `NetworkError::ResourceExhaustion` because we pre-fill the
    /// bounded channel with a dummy item, leaving no space.
    ///
    /// Returns `(WorkerPool, Receiver<TaskItem<'threads, T>>)`.
    /// - Keep `_rx` in the test scope so the channel doesn't close.
    /// - Because the channel is always *full*, any further send => ResourceExhaustion.
    #[cfg(test)]
    pub fn new_test_dummy_resource_exhaustion()
        -> Result<(Self, tokio::sync::mpsc::Receiver<TaskItem<'threads, T>>), NetworkError>
    {
        use tokio::sync::mpsc;

        // Create a channel of size=1
        let (main_tasks_tx, main_tasks_rx) = mpsc::channel::<TaskItem<'threads, T>>(1);

        // Also create a results channel, though we won't fill it
        let (results_tx, results_rx) = mpsc::channel::<TaskResult>(1);

        let dummy_threads = Vec::new();

        // Build the WorkerPool
        let pool = WorkerPoolBuilder::default()
            .main_tasks_tx(main_tasks_tx.clone())
            .threads(dummy_threads)
            .results_rx(AsyncMutex::new(results_rx))
            .results_tx_for_test(Some(results_tx))   // <--- KEY
            .build()
            .unwrap();

        // Fill the channel with one "dummy" TaskItem, so it is at capacity.
        // Any subsequent call to `.submit(...)` must do `send(...)`, which will fail with a
        // channel-full error => ResourceExhaustion in your `map_err(...)`.
        let shared_in_degs  = Arc::new(AsyncMutex::new(vec![]));
        let network         = Arc::new(AsyncMutex::new(Network::default()));
        let child_nodes_tx  = mpsc::channel::<usize>(1).0;
        let ready_nodes_tx  = mpsc::channel::<usize>(1).0;
        let completed_nodes = SharedCompletedNodes::new();

        let dummy_task = task_item!(
            node_idx:        9999_usize,
            permit:          None,
            network:         network.clone(),
            shared_in_degs:  shared_in_degs.clone(),
            output_tx:       None,
            checkpoint_cb:   None,
            child_nodes_tx:  child_nodes_tx.clone(),
            ready_nodes_tx:  ready_nodes_tx.clone(),
            completed_nodes: completed_nodes.clone()
        );

        // We use a **non-async** try_send to fill the buffer.
        // This item stays in the channel forever (no aggregator to read it).
        main_tasks_tx.try_send(dummy_task)
            .map_err(|_err| NetworkError::ResourceExhaustion {
                resource: "WorkerPool Main Tasks Channel (pre-filled)".into()
            })?;

        // Return the WorkerPool plus the main_tasks_rx so the test can keep the channel open.
        Ok((pool, main_tasks_rx))
    }

    /// Helper that inserts the given results into `pool.results_rx` so that
    /// `try_recv_result()` yields them in that order, then returns None.
    #[cfg(test)]
    pub fn push_results_into_worker_pool(
        &self,
        results: Vec<TaskResult>,
    ) {
        if let Some(ref results_tx_for_test) = self.results_tx_for_test {
            for r in results {
                // Non-blocking. If the channel is full, this returns an error.
                results_tx_for_test
                    .try_send(r)
                    .expect("Failed to push TaskResult (channel full?)");
                }
        }
    }
}
