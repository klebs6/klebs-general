// ---------------- [ File: hydro2-async-scheduler/src/spawn_aggregator_thread_and_workers.rs ]
crate::ix!();

/// Spawns an aggregator plus N worker threads, returning a Vec of handles.
///
/// - `main_tasks_rx`: aggregator reads tasks from this channel.
/// - `worker_senders`: aggregator will send tasks to these N worker channels.
/// - `worker_receivers`: each worker reads from its channel, parallel to aggregator
/// - `results_tx`: all workers share this to send `TaskResult` to aggregator/test code.
/// 
/// Return: `Vec<ScopedJoinHandle<'scope, ()>>` so you can join or store them.
pub fn spawn_aggregator_and_workers<'scope, T>(
    scope:            &'scope Scope<'scope, '_>,
    main_tasks_rx:    Receiver<TaskItem<'scope, T>>,
    worker_senders:   Vec<Sender<TaskItem<'scope, T>>>,
    worker_receivers: Vec<Receiver<TaskItem<'scope, T>>>,
    results_tx:       Sender<TaskResult>,
) -> Vec<ScopedJoinHandle<'scope, ()>>
where
    T: std::fmt::Debug + Send + Sync + 'scope,
{
    // (A) Spawn aggregator thread
    let aggregator_handle = spawn_aggregator(
        scope, 
        main_tasks_rx,
        worker_senders
    );

    eprintln!("spawn_aggregator_and_workers => aggregator thread spawned => handle pushed");

    // Collect aggregator handle + worker handles
    let mut threads = vec![aggregator_handle];

    // (B) Spawn each worker thread
    for (worker_id, worker_rx) in worker_receivers.into_iter().enumerate() {
        // Possibly clone or pass anything needed to the worker
        let results_tx_clone = results_tx.clone();

        let handle = spawn_worker_thread(
            scope,
            worker_id,
            worker_rx,
            results_tx_clone
        );

        threads.push(handle);

        eprintln!(
            "spawn_aggregator_and_workers => worker #{} thread spawned => handle pushed",
            worker_id
        );
    }

    threads
}

#[cfg(test)]
mod spawn_aggregator_and_workers_tests {
    use super::*;

    //===========================================================
    // Existing Basic Tests
    //===========================================================
    #[test]
    fn test_spawn_aggregator_and_workers_no_workers() {
        thread::scope(|scope| {
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(1);
            drop(main_tx); // aggregator sees no tasks
            let worker_senders   = vec![];
            let worker_receivers = vec![];

            let (res_tx, mut res_rx) = channel::<TaskResult>(1);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx,
            );
            for handle in handles {
                handle.join().expect("Thread panicked");
            }

            // aggregator => no tasks => no TaskResult => ...
            assert!(res_rx.try_recv().is_err());
        });
    }

    #[test]
    fn test_spawn_aggregator_and_workers_no_tasks_2workers() {
        thread::scope(|scope| {
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(1);
            drop(main_tx); // aggregator sees no tasks => none
            let (w1_tx, w1_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(1);
            let (w2_tx, w2_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(1);

            let worker_senders   = vec![w1_tx, w2_tx];
            let worker_receivers = vec![w1_rx, w2_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(10);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx
            );
            for h in handles {
                h.join().expect("Thread panicked");
            }
            assert!(res_rx.try_recv().is_err());
        });
    }

    #[test]
    fn test_spawn_aggregator_and_workers_with_tasks() {
        thread::scope(|scope| {
            let (main_tx, main_rx) = channel::<TaskItem<'_,TestWireIO<i32>>>(10);
            let (w1_tx, w1_rx) = channel::<TaskItem<'_,TestWireIO<i32>>>(10);
            let (w2_tx, w2_rx) = channel::<TaskItem<'_,TestWireIO<i32>>>(10);

            let worker_senders   = vec![w1_tx, w2_tx];
            let worker_receivers = vec![w1_rx, w2_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(10);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx
            );

            // Send 3 tasks
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..3 {
                    let task = mock_minimal_task_item_with_permit(i as usize);
                    main_tx.send(task).await.unwrap();
                }
                drop(main_tx);
            });

            for h in handles {
                h.join().expect("Thread panicked");
            }

            // Possibly we see some TaskResults
            let mut count = 0;
            while let Ok(tres) = res_rx.try_recv() {
                count += 1;
                // do more checks here
            }
            // We might see 3 results if each worker produces a result
            // ...
        });
    }

    //===========================================================
    // 1) Concurrency with Heavy Load
    //===========================================================
    #[test]
    fn test_concurrency_heavy_load() {
        thread::scope(|scope| {
            // aggregator capacity=5, but we send more tasks, e.g. 20
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);

            // Suppose we have 2 workers
            let (w1_tx, w1_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);
            let (w2_tx, w2_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);

            let worker_senders   = vec![w1_tx, w2_tx];
            let worker_receivers = vec![w1_rx, w2_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(20);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx,
            );

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..20 {
                    let t = mock_minimal_task_item_with_permit(i);
                    main_tx.send(t).await.unwrap();
                }
                drop(main_tx);
            });

            // aggregator sees 20 tasks => dispatch => workers handle them
            for h in handles {
                h.join().expect("Thread panicked");
            }

            // Now read results
            let mut count = 0;
            while let Ok(_tr) = res_rx.try_recv() {
                count += 1;
            }
            // We might expect 20 results if each task produces a single result
            // Or fewer if some tasks produce errors, or aggregator logic is different
            assert_eq!(count, 20, "Expected each of 20 tasks => 1 result");
        });
    }

    //===========================================================
    // 2) Channel Closure Midway
    //===========================================================
    #[test]
    fn test_channel_closure_midway() {
        thread::scope(|scope| {
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(10);

            let (w1_tx, w1_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(10);
            let (w2_tx, w2_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(10);

            let worker_senders   = vec![w1_tx, w2_tx];
            let worker_receivers = vec![w1_rx, w2_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(10);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx
            );

            // We'll send a few tasks, then forcibly close aggregator's channel
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..5 {
                    let t = mock_minimal_task_item_with_permit(i);
                    main_tx.send(t).await.unwrap();
                }
                // Now forcibly close aggregator channel => aggregator sees None => stops
                drop(main_tx);
            });

            // aggregator sees tasks 0..4 => dispatch => workers possibly produce results
            // aggregator sees None => returns => let's join
            for h in handles {
                h.join().expect("Thread panicked");
            }

            // read results
            let mut results = Vec::new();
            while let Ok(r) = res_rx.try_recv() {
                results.push(r);
            }
            // Possibly we see 5 results => or less, depending on aggregator logic
            assert_eq!(results.len(), 5, "We expected 5 tasks => 5 results (assuming no errors).");
        });
    }

    //===========================================================
    // 3) Error Injection
    //===========================================================
    #[test]
    fn test_error_injection() {
        thread::scope(|scope| {
            // aggregator channel
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);
            // 2 worker channels
            let (w1_tx, w1_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);
            let (w2_tx, w2_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);

            let worker_senders   = vec![w1_tx, w2_tx];
            let worker_receivers = vec![w1_rx, w2_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(10);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx
            );

            // We'll send tasks that cause worker to fail => e.g. node_idx=999 => out of bounds => error
            // or use a failing operator
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // normal
                let normal_task = mock_minimal_task_item_with_permit(0);
                main_tx.send(normal_task).await.unwrap();

                // out-of-bounds => aggregator or worker sees error
                let failing_task = mock_minimal_task_item_with_permit(999); // no network node => error
                main_tx.send(failing_task).await.unwrap();

                // or if you have FailingOperator => node=1 => failing operator
                let operator_fail = mock_failing_operator_task(1, "some reason");
                main_tx.send(operator_fail).await.unwrap();

                drop(main_tx);
            });

            for h in handles {
                h.join().expect("Thread panicked");
            }

            // read results => we might see normal=Ok, 999=InvalidNode, 1=OperatorFailed
            let mut normal_count = 0;
            let mut error_count  = 0;
            while let Ok(tr) = res_rx.try_recv() {
                if tr.error().is_none() {
                    normal_count += 1;
                } else {
                    error_count += 1;
                }
            }
            // We expect 1 normal, 2 errors => total 3
            assert_eq!(normal_count + error_count, 3);
            assert_eq!(normal_count, 1);
            assert_eq!(error_count, 2);
        });
    }

    //===========================================================
    // 4) Checkpoint Callback Example
    //===========================================================
    #[test]
    fn test_checkpoint_callback() {
        thread::scope(|scope| {
            // aggregator + 1 worker to keep it simple
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);
            let (w_tx, w_rx)       = channel::<TaskItem<'_, TestWireIO<i32>>>(5);

            let worker_senders   = vec![w_tx];
            let worker_receivers = vec![w_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(5);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx,
            );

            // We'll define a mock checkpoint callback that appends to a global or Arc<AsyncMutex> vector for verification
            // Or you embed it in mock_minimal_task_item_with_permit
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..3 {
                    let t = mock_task_with_checkpoint(i);
                    main_tx.send(t).await.unwrap();
                }
                drop(main_tx);
            });

            for h in handles {
                h.join().expect("Thread panicked");
            }

            // read results
            let mut results = vec![];
            while let Ok(r) = res_rx.try_recv() {
                results.push(r);
            }
            assert_eq!(results.len(), 3);

            // Now check that the checkpoint was invoked for each node
            // Possibly read from a static or from your callback struct
            let checkpoint_data = get_mock_checkpoint_invocations();
            // ensure len=3, correct order, etc.
            assert_eq!(checkpoint_data, vec![0,1,2]);
        });
    }

    //===========================================================
    // 5) Freed Children scenario
    //===========================================================
    #[disable]
    #[test]
    fn test_freed_children() {
        thread::scope(|scope| {
            let (main_tx, main_rx) = channel::<TaskItem<'_, TestWireIO<i32>>>(5);
            let (w1_tx, w1_rx)     = channel::<TaskItem<'_, TestWireIO<i32>>>(5);

            let worker_senders   = vec![w1_tx];
            let worker_receivers = vec![w1_rx];

            let (res_tx, mut res_rx) = channel::<TaskResult>(10);

            let handles = spawn_aggregator_and_workers(
                scope,
                main_rx,
                worker_senders,
                worker_receivers,
                res_tx,
            );

            // We'll send a task that frees child node=999 => aggregator or logic sees it => aggregator re-queues?
            // Or if aggregator doesn't handle Freed children, we just ensure worker tries sending Freed child=999
            // Then aggregator is done => see partial Freed logic

            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let t = mock_task_that_frees_child(0, vec![999]);
                main_tx.send(t).await.unwrap();
                drop(main_tx);
            });

            for h in handles {
                h.join().expect("Thread panicked");
            }

            // aggregator might re-queue Freed child or produce some results
            // check the res_rx
            let mut results = Vec::new();
            while let Ok(r) = res_rx.try_recv() {
                results.push(r);
            }
            // Freed child might produce a new TaskResult or aggregator might do nothing
            // depends on your aggregator logic
            // ...
        });
    }
}
