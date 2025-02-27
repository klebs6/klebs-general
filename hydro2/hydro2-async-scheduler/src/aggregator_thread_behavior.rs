// ---------------- [ File: hydro2-async-scheduler/src/aggregator_thread_behavior.rs ]
crate::ix!();

/// Aggregator thread that reads incoming `TaskItem`s, distributing them to workers.
/// We now log each phase of the aggregator's loop in more detail, which can
/// reveal if aggregator is stuck waiting for input or if workers are closed prematurely.
pub async fn aggregator_thread_behavior<'threads, T>(
    mut main_tasks_rx: tokio::sync::mpsc::Receiver<TaskItem<'threads, T>>,
    worker_senders: Vec<tokio::sync::mpsc::Sender<TaskItem<'threads, T>>>,
)
where
    T: Debug + Send + Sync + 'threads,
{
    eprintln!(
        "AGGREGATOR => ENTER => aggregator_thread_behavior => worker_senders.len()={}",
        worker_senders.len()
    );

    let mut rr_counter = 0usize;
    loop {
        eprintln!("AGGREGATOR => waiting to recv next TaskItem from main_tasks_rx");
        match main_tasks_rx.recv().await {
            Some(task) => {
                let node_idx = *task.node_idx();
                eprintln!(
                    "AGGREGATOR => got TaskItem => node_idx={}, round_robin_index={}",
                    node_idx,
                    rr_counter
                );

                let worker_idx = rr_counter % worker_senders.len();
                eprintln!(
                    "AGGREGATOR => sending node_idx={} => worker #{}",
                    node_idx,
                    worker_idx
                );

                if let Err(send_err) = worker_senders[worker_idx].send(task).await {
                    eprintln!(
                        "AGGREGATOR => ERROR sending to worker #{} => node_idx={} => err={:?}",
                        worker_idx,
                        node_idx,
                        send_err
                    );
                }

                rr_counter += 1;
            }
            None => {
                eprintln!(
                    "AGGREGATOR => main_tasks_rx closed => aggregator done => dropping all worker_senders"
                );
                break;
            }
        }
    }

    // Drop each worker’s Sender so they exit
    for (widx, tx) in worker_senders.into_iter().enumerate() {
        eprintln!("AGGREGATOR => dropping worker_senders[{}]", widx);
        drop(tx);
    }

    eprintln!("AGGREGATOR => aggregator_thread_behavior => DONE => returning");
}

/// Tests for `aggregator_thread_behavior`.
///
/// These tests demonstrate how to fix the type inference issue by
/// explicitly annotating the channel to send/receive
/// `TaskItem<'static, T>`.
#[cfg(test)]
mod aggregator_thread_behavior_tests {
    use super::*;

    /// Helper to create arcs/mutexes for the test `TaskItem` fields.
    fn create_shared_data() -> (
        Arc<AsyncMutex<Network<TestWireIO<i32>>>>,
        Arc<AsyncMutex<Vec<usize>>>,
        SharedCompletedNodes,
        mpsc::Sender<usize>,
        mpsc::Sender<usize>,
    ) {
        let network             = Arc::new(AsyncMutex::new(Network::<TestWireIO<i32>>::default()));
        let shared_in_degs      = Arc::new(AsyncMutex::new(vec![]));
        let completed_nodes     = SharedCompletedNodes::new();
        let (child_nodes_tx, _) = mpsc::channel(10);
        let (ready_nodes_tx, _) = mpsc::channel(10);

        (network, shared_in_degs, completed_nodes, child_nodes_tx,ready_nodes_tx)
    }

    /// This test ensures that tasks are distributed to workers and
    /// received properly in a round-robin fashion.
    #[traced_test]
    async fn test_aggregator_thread_behavior_distributes_tasks() {
        let num_workers = 3;
        let buffer_size = 10;

        // 1) Create the worker channels
        let (worker_senders, mut worker_receivers): (
            Vec<mpsc::Sender<TaskItem<'static, TestWireIO<i32>>>>,
            Vec<mpsc::Receiver<TaskItem<'static, TestWireIO<i32>>>>
        ) = (0..num_workers)
            .map(|_| mpsc::channel(buffer_size))
            .unzip();

        // 2) Create the aggregator’s main tasks channel
        let (main_tasks_tx, main_tasks_rx): (mpsc::Sender<_>, mpsc::Receiver<_>) =
            mpsc::channel(buffer_size);

        // 3) Create *both* child_nodes AND ready_nodes channels
        let (child_nodes_tx, _child_nodes_rx) = mpsc::channel::<usize>(10);
        let (ready_nodes_tx, _ready_nodes_rx) = mpsc::channel::<usize>(10);
        // `_child_nodes_rx`, `_ready_nodes_rx` unused if aggregator doesn't read them.

        // 4) Build shared data for tasks
        let network             = Arc::new(AsyncMutex::new(Network::<TestWireIO<i32>>::default()));
        let shared_in_degs      = Arc::new(AsyncMutex::new(vec![]));
        let completed_nodes     = SharedCompletedNodes::new();

        // 5) Send tasks
        for i in 0_usize..6 {
            let task = task_item!(
                node_idx:        i,
                permit:          None,
                network:         network.clone(),
                shared_in_degs:  shared_in_degs.clone(),
                output_tx:       None,
                checkpoint_cb:   None,
                child_nodes_tx:  child_nodes_tx.clone(),
                ready_nodes_tx:  ready_nodes_tx.clone(),  // <--- MUST define earlier
                completed_nodes: completed_nodes.clone()
            );
            main_tasks_tx.send(task).await.expect("Failed to send task");
        }
        drop(main_tasks_tx);

        // 6) aggregator_thread_behavior => reads from main_tasks_rx => sends to worker_senders
        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        // 7) Check that each worker got tasks
        let mut received_tasks = Vec::new();
        for mut rx in worker_receivers {
            while let Some(task) = rx.recv().await {
                received_tasks.push(*task.node_idx());
            }
        }
        received_tasks.sort_unstable();
        assert_eq!(received_tasks, vec![0,1,2,3,4,5]);
    }

    /// This test checks that worker channels are closed (returning `None`)
    /// after the aggregator finishes distributing tasks.
    #[traced_test]
    async fn test_aggregator_thread_behavior_closes_worker_channels() {
        let num_workers = 2;
        let buffer_size = 5;

        // Again, explicitly specify the type
        let (worker_senders, mut worker_receivers): (
            Vec<mpsc::Sender<TaskItem<'static, TestWireIO<i32>>>>,
            Vec<mpsc::Receiver<TaskItem<'static, TestWireIO<i32>>>>
        ) = (0..num_workers)
            .map(|_| mpsc::channel(buffer_size))
            .unzip();

        // Main tasks channel
        let (main_tasks_tx, main_tasks_rx): (
            mpsc::Sender<TaskItem<'static, TestWireIO<i32>>>,
            mpsc::Receiver<TaskItem<'static, TestWireIO<i32>>>,
        ) = mpsc::channel(buffer_size);

        // Immediately drop the main tasks sender so aggregator sees no tasks
        drop(main_tasks_tx);

        // Run the aggregator, which should exit quickly
        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        // Confirm each worker channel is closed (i.e., `recv()` returns `None`)
        for mut rx in worker_receivers {
            assert!(
                rx.recv().await.is_none(),
                "Channel should be closed and return None"
            );
        }
    }

    /// 1. Test aggregator with a single worker.
    ///    All tasks should go to that single worker, in order.
    #[traced_test]
    async fn test_single_worker_all_tasks_go_to_one_worker() {
        let num_workers = 1;
        let buffer_size = 10;
        let (worker_senders, mut worker_receivers) = create_worker_channels(num_workers, buffer_size);

        let (main_tasks_tx, main_tasks_rx) =
            mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(buffer_size);

        // Create child + ready channels:
        let (network, shared_in_degs, completed_nodes, child_nodes_tx, ready_nodes_tx) = create_shared_data(); 

        for i in 0_usize..5 {
            let task = task_item!(
                node_idx:        i,
                permit:          None,
                network:         network.clone(),
                shared_in_degs:  shared_in_degs.clone(),
                output_tx:       None,
                checkpoint_cb:   None,
                child_nodes_tx:  child_nodes_tx.clone(),
                ready_nodes_tx:  ready_nodes_tx.clone(),
                completed_nodes: completed_nodes.clone()
            );
            main_tasks_tx.send(task).await.unwrap();
        }
        drop(main_tasks_tx);

        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        let mut received = Vec::new();
        while let Some(task) = worker_receivers[0].recv().await {
            received.push(*task.node_idx());
        }
        assert_eq!(received, vec![0,1,2,3,4]);
    }

    //#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[traced_test]
    async fn test_more_tasks_than_workers() {

        eprintln!("TEST => ENTER: test_more_tasks_than_workers");
        let num_workers = 3;
        let buffer_size = 5;

        // Create bounded worker channels
        eprintln!(
            "TEST => Creating worker channels: num_workers={}, buffer_size={}",
            num_workers, buffer_size
        );
        let (worker_senders, worker_receivers) = create_worker_channels(num_workers, buffer_size);

        // Create main tasks channel
        eprintln!("TEST => Creating main tasks channel: buffer_size={}", buffer_size);
        let (main_tasks_tx, main_tasks_rx) =
            mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(buffer_size);

        // Prepare data for tasks
        let (network, shared_in_degs, completed_nodes, child_nodes_tx, ready_nodes_tx) = create_shared_data();

        // -------------------------------------------------------------------------
        // 1) SPIN UP THE AGGREGATOR TASK **BEFORE** WE SEND TASKS
        // -------------------------------------------------------------------------
        eprintln!("TEST => SPAWNING aggregator task (so it can start draining ASAP)");
        let aggregator_handle = tokio::spawn(async move {
            eprintln!("AGGREGATOR => aggregator_thread_behavior launched");
            aggregator_thread_behavior(main_tasks_rx, worker_senders).await;
            eprintln!("AGGREGATOR => aggregator_thread_behavior done");
        });

        // -------------------------------------------------------------------------
        // 2) NOW SEND TASKS — aggregator is already running to drain the channel
        // -------------------------------------------------------------------------
        let total_tasks = 10;
        eprintln!("TEST => SENDING {} tasks to aggregator", total_tasks);
        for i in 0..total_tasks {
            let task = task_item!(
                node_idx:        i,
                permit:          None,
                network:         network.clone(),
                shared_in_degs:  shared_in_degs.clone(),
                output_tx:       None,
                checkpoint_cb:   None,
                child_nodes_tx:  child_nodes_tx.clone(),
                ready_nodes_tx:  ready_nodes_tx.clone(),
                completed_nodes: completed_nodes.clone()
            );
            eprintln!("TEST => Sending task i={}", i);
            // The aggregator is already alive, so once the channel is full,
            // it can read tasks and make room for more.
            main_tasks_tx.send(task).await.unwrap();
        }
        eprintln!("TEST => Dropping main_tasks_tx so aggregator sees no more tasks");
        drop(main_tasks_tx);

        // -------------------------------------------------------------------------
        // 3) WORKER TASKS: read from each worker receiver
        // -------------------------------------------------------------------------
        eprintln!("TEST => SPAWNING worker tasks to concurrently read from channels");
        let mut worker_handles = Vec::new();
        for (idx, mut rx) in worker_receivers.into_iter().enumerate() {
            let handle = tokio::spawn(async move {
                let mut out = Vec::new();
                loop {
                    match rx.recv().await {
                        Some(task) => {
                            eprintln!(
                                "WORKER #{} => GOT task node_idx={}",
                                idx,
                                task.node_idx()
                            );
                            out.push(*task.node_idx());
                        }
                        None => {
                            eprintln!("WORKER #{} => channel closed => EXIT worker loop", idx);
                            break;
                        }
                    }
                }
                out
            });
            worker_handles.push(handle);
        }

        // -------------------------------------------------------------------------
        // 4) WAIT FOR AGGREGATOR + WORKER TASKS TO COMPLETE
        // -------------------------------------------------------------------------
        eprintln!("TEST => WAIT for aggregator and worker tasks to complete");
        let worker_res_vec = futures::future::join_all(worker_handles);
        let (agg_res, worker_res_vec) = tokio::join!(aggregator_handle, worker_res_vec);
        eprintln!("TEST => aggregator + workers have joined");

        agg_res.expect("Aggregator panicked");

        // Collect from all workers
        let mut all_received = Vec::new();
        for (widx, worker_res) in worker_res_vec.into_iter().enumerate() {
            match worker_res {
                Ok(mut chunk) => {
                    eprintln!(
                        "TEST => Worker #{} returned {} tasks: {:?}",
                        widx,
                        chunk.len(),
                        chunk
                    );
                    all_received.append(&mut chunk);
                }
                Err(e) => panic!("Worker task #{} panicked: {e}", widx),
            }
        }

        eprintln!("TEST => All worker tasks done, sorting results");
        all_received.sort_unstable();
        eprintln!("TEST => final received = {:?}", all_received);

        assert_eq!(
            all_received,
            (0..total_tasks).collect::<Vec<_>>(),
            "All tasks 0..9 should be distributed among workers"
        );
        eprintln!("TEST => PASS: test_more_tasks_than_workers");
    }

    /// 3. Test aggregator with fewer tasks than workers (e.g., 2 tasks, 5 workers).
    ///    Some workers will receive no tasks.
    #[traced_test]
    async fn test_fewer_tasks_than_workers() {
        let num_workers = 5;
        let buffer_size = 5;
        let (worker_senders, mut worker_receivers) = create_worker_channels(num_workers, buffer_size);
        let (main_tasks_tx, main_tasks_rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(buffer_size);

        let (network, shared_in_degs, completed_nodes, child_nodes_tx, ready_nodes_tx) = create_shared_data();

        // Send only 2 tasks
        for i in 0_usize..2 {
            let task = task_item!(
                node_idx:        i,
                permit:          None,
                network:         network.clone(),
                shared_in_degs:  shared_in_degs.clone(),
                output_tx:       None,
                checkpoint_cb:   None,
                child_nodes_tx:  child_nodes_tx,
                ready_nodes_tx:  ready_nodes_tx,
                completed_nodes: completed_nodes.clone()
            );
            main_tasks_tx.send(task).await.expect("Failed to send task");
        }
        drop(main_tasks_tx);

        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        // Gather what each worker got
        let mut all_received: Vec<Vec<usize>> = vec![];
        for mut rx in worker_receivers {
            let mut chunk = vec![];
            while let Some(task) = rx.recv().await {
                chunk.push(*task.node_idx());
            }
            all_received.push(chunk);
        }

        // Flatten & verify
        let flattened = all_received.iter().flatten().cloned().collect::<Vec<_>>();
        assert_eq!(flattened.len(), 2, "We only sent 2 tasks total");
        assert_eq!(flattened, vec![0, 1], "Expected tasks 0 and 1");
    }

    /// 4. Test aggregator with immediate drop of main sender (no tasks).
    ///    Aggregator should exit immediately and close worker channels.
    #[traced_test]
    async fn test_no_tasks_main_sender_drop_immediately() {
        let num_workers = 4;
        let buffer_size = 5;
        let (worker_senders, mut worker_receivers) = create_worker_channels(num_workers, buffer_size);

        // Drop main immediately
        let (_, main_tasks_rx) = mpsc::channel::<TaskItem<'static, i32>>(buffer_size);

        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        // Confirm worker channels are closed (none got tasks).
        for mut rx in worker_receivers {
            assert!(rx.recv().await.is_none(), "Expected worker channel to be closed");
        }
    }

    /// 5. Test aggregator with multiple concurrent sends.
    ///    We spawn several async tasks that each send some tasks to the aggregator.
    ///    Then we close the channel. We ensure aggregator processes them all.
    #[traced_test]
    async fn test_multiple_concurrent_sends() {
        let num_workers = 3;
        let buffer_size = 50;
        let (worker_senders, mut worker_receivers) = create_worker_channels(num_workers, buffer_size);
        let (main_tasks_tx, main_tasks_rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(buffer_size);

        let (network, shared_in_degs, completed_nodes, child_nodes_tx, ready_nodes_tx) = create_shared_data();

        // We'll spawn 5 concurrent tasks, each sending 10 tasks (50 total).
        let mut send_futures = vec![];
        for batch_id in 0_usize..5 {

            let tx_clone       = main_tasks_tx.clone();
            let net_clone      = network.clone();
            let degs_clone     = shared_in_degs.clone();
            let comp_clone     = completed_nodes.clone();
            let child_tx_clone = child_nodes_tx.clone();
            let ready_tx_clone = ready_nodes_tx.clone();

            send_futures.push(tokio::spawn(async move {
                for i in 0_usize..10 {
                    let global_idx = batch_id * 10 + i;
                    let task = task_item!(
                        node_idx:        global_idx,
                        permit:          None,
                        network:         net_clone.clone(),
                        shared_in_degs:  degs_clone.clone(),
                        output_tx:       None,
                        checkpoint_cb:   None,
                        child_nodes_tx:  child_tx_clone.clone(),
                        ready_nodes_tx:  ready_tx_clone.clone(),
                        completed_nodes: comp_clone.clone()
                    );
                    tx_clone.send(task).await.expect("Failed to send task");
                }
            }));
        }

        // Wait for all sending tasks to finish, then drop main
        join_all(send_futures).await;
        drop(main_tasks_tx);

        aggregator_thread_behavior(main_tasks_rx, worker_senders).await;

        // Verify that all tasks (0..49) arrived at the workers
        let mut all_received = vec![];
        for rx in worker_receivers.iter_mut() {
            while let Some(task) = rx.recv().await {
                all_received.push(*task.node_idx());
            }
        }

        all_received.sort();
        let expected: Vec<usize> = (0..50).collect();
        assert_eq!(all_received, expected, "All 50 tasks should have been processed");
    }

    // Additional edge cases could include:
    // - Checking aggregator reaction to worker send failures,
    // - Testing for concurrency constraints if using semaphore permits,
    // - Integration with actual processing logic in the worker, etc.
    //
}
