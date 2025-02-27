// ---------------- [ File: hydro2-async-scheduler/src/worker_pool.rs ]
crate::ix!();

/// A pool of OS threads: 
/// - 1 aggregator thread reading from `main_tasks_rx` (single consumer),
/// - N worker threads, each with its own private channel,
/// - A results channel for `TaskResult`.
#[derive(Builder)]
#[builder(setter(into), pattern = "owned")]
pub struct WorkerPool<'threads, T>
where
    T: Debug + Send + Sync + 'threads
{
    /// For sending tasks into aggregator
    main_tasks_tx: Sender<TaskItem<'threads, T>>,

    /// Threads for aggregator + workers
    threads: Vec<ScopedJoinHandle<'threads, ()>>,

    /// For receiving TaskResult from all workers
    results_rx: AsyncMutex<Receiver<TaskResult>>,

    #[cfg(test)]
    #[builder(default)]
    pub(crate) results_tx_for_test: Option<Sender<TaskResult>>
}

impl<'threads, T> WorkerPool<'threads, T>
where
    T: Debug + Send + Sync + 'threads,
{
    /// Build the aggregator + N workers within a synchronous scope.
    /// - The aggregator (thread) reads from `main_tasks_rx` (single consumer).
    /// - It fans out tasks to each worker’s private channel in round-robin.
    /// - Each worker runs in a separate OS thread, with its own mini tokio runtime.
    /// - We also have one results channel so the aggregator can send back TaskResult items,
    ///   which an external consumer (like `process_immediate`) can poll via `try_recv_result`.
    ///
    /// The aggregator closes each worker’s channel at the end, ensuring that idle workers
    /// (which never receive tasks) also exit cleanly.
    pub fn new_in_scope(
        scope: &'threads Scope<'threads, '_>,
        num_workers: usize,
        buffer_size: usize,
    ) -> Self {

        eprintln!(
            "WorkerPool::new_in_scope => setting up aggregator + {} workers, buffer_size={}",
            num_workers, buffer_size
        );

        //=== (A) Main tasks channel => aggregator is the single consumer
        let (main_tasks_tx, main_tasks_rx) = mpsc::channel::<TaskItem<'threads, T>>(buffer_size);
        eprintln!("WorkerPool::new_in_scope => created main_tasks channel (aggregator consumer)");

        //=== (B) A results channel for all workers => external code can poll results
        let (results_tx, results_rx) = mpsc::channel::<TaskResult>(buffer_size);
        eprintln!("WorkerPool::new_in_scope => created results channel for all workers");

        //=== (C) Worker channels: each worker has its own channel
        // aggregator will send tasks to these
        let (worker_senders, worker_receivers) = create_worker_channels(num_workers, buffer_size);

        // aggregator + N workers => total of num_workers + 1 threads
        let threads = spawn_aggregator_and_workers(
            scope,
            main_tasks_rx,
            worker_senders,
            worker_receivers,
            results_tx
        );

        eprintln!("WorkerPool::new_in_scope => aggregator + {} workers => returning WorkerPool", num_workers);

        WorkerPool {
            main_tasks_tx,
            threads,
            results_rx: AsyncMutex::new(results_rx),

            #[cfg(test)]
            results_tx_for_test: None
        }
    }

    /// Submit a task => aggregator picks it up, fans out to a worker.
    pub async fn submit(&self, item: TaskItem<'threads, T>) -> Result<(), NetworkError> {
        eprintln!("WorkerPool::submit => sending to aggregator main_tasks channel => node_idx={}", item.node_idx());
        // Instead of .send(...).await, do a non-blocking try_send:
        match self.main_tasks_tx.try_send(item) {
            Ok(()) => Ok(()),
            Err(_e) => Err(NetworkError::ResourceExhaustion {
                resource: "WorkerPool Main Tasks Channel".into(),
            }),
        }
    }

    /// Non-blocking poll of the results channel from workers
    pub async fn try_recv_result(&self) -> Option<TaskResult> {
        let mut guard = self.results_rx.lock().await;
        let res = guard.try_recv().ok();
        if let Some(ref r) = res {
            eprintln!("WorkerPool::try_recv_result => got a result => node_idx={}", r.node_idx());
        }
        res
    }

    pub fn is_main_channel_closed(&self) -> bool {
        self.main_tasks_tx.is_closed()
    }

    /// Force aggregator to see "None" => aggregator returns => shuts down
    pub fn close_main_tasks_channel(&self) {
        eprintln!("WorkerPool::close_main_tasks_channel => about to drop main_tasks_tx");
        eprintln!("Pointer: {:p}", &self.main_tasks_tx);
        drop(&self.main_tasks_tx);
        eprintln!("WorkerPool::close_main_tasks_channel => after drop(main_tasks_tx)");
    }

    /// Shut down everything: aggregator + workers 
    /// (if aggregator is still open, close it, then join threads).
    pub fn shutdown(self) {
        eprintln!("WorkerPool::shutdown => dropping main_tasks_tx => aggregator sees None => eventually done");
        drop(self.main_tasks_tx);

        for (i, th) in self.threads.into_iter().enumerate() {
            eprintln!("WorkerPool::shutdown => joining aggregator/worker thread #{}", i);
            let _ = th.join();
        }
        eprintln!("WorkerPool::shutdown => all aggregator+worker threads joined => done");
    }
}
