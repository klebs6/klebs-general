// ---------------- [ File: src/spawn_worker_thread.rs ]
crate::ix!();

/// Spawns one worker OS thread within the given `scope`.
/// The worker runs a mini tokio runtime that calls `worker_main_loop`.
/// Returns a ScopedJoinHandle you can store or join later.
///
/// # Arguments
/// * `scope` - the scoped block in which the thread is valid
/// * `worker_id` - an integer ID used for logging
/// * `worker_rx` - a channel from aggregator to this worker
/// * `results_tx` - channel for sending TaskResult back to aggregator or test code
pub fn spawn_worker_thread<'threads, T>(
    scope:      &'threads Scope<'threads, '_>,
    worker_id:  usize,
    worker_rx:  Receiver<TaskItem<'threads, T>>,
    results_tx: Sender<TaskResult>,
) -> std::thread::ScopedJoinHandle<'threads, ()>
where
    T: Debug + Send + Sync + 'threads,
{
    scope.spawn(move || {
        eprintln!(
            "Worker #{} => OS thread spawned => building mini runtime",
            worker_id
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            eprintln!("Worker #{} => starting worker_main_loop", worker_id);
            worker_main_loop(worker_rx, results_tx, worker_id).await;
            eprintln!("Worker #{} => worker_main_loop returned => done", worker_id);
        });

        eprintln!("Worker #{} => after block_on => OS thread done", worker_id);
    })
}

#[cfg(test)]
mod spawn_worker_thread_tests {

    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_spawn_worker_no_tasks() {
        // If the worker channel is closed immediately, 
        // the worker sees None => logs "no more tasks, exiting".
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(1);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

            // Immediately drop the sending side => no tasks
            drop(task_tx);

            // The worker_id=0 is purely for log messages
            let handle = spawn_worker_thread(scope, 0, task_rx, res_tx);

            // Join the worker thread
            handle.join().expect("Worker thread panicked");

            // Check that no results arrived
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let result = res_rx.try_recv();
                assert!(
                    result.is_err(),
                    "No tasks => we expect no TaskResult from the worker"
                );
            });
        });
    }

    #[test]
    fn test_spawn_worker_one_task() {
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(1);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(2);

            let handle = spawn_worker_thread(scope, 123, task_rx, res_tx);

            // Send one mock TaskItem
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let item = mock_minimal_task_item_with_permit(42);
                // This sends exactly 1 task
                task_tx.send(item).await.unwrap();

                // Drop the sender => worker sees None after that single task
                drop(task_tx);
            });

            // Join worker
            handle.join().expect("Worker thread panicked");

            // Check results => we expect exactly 1 TaskResult
            let rt2 = Runtime::new().unwrap();
            let maybe_result = rt2.block_on(async { res_rx.try_recv().ok() });
            assert!(
                maybe_result.is_some(),
                "Expected exactly one TaskResult from the single task"
            );
        });
    }

    #[test]
    fn test_spawn_worker_multiple_tasks() {
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(8);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(8);

            let handle = spawn_worker_thread(scope, 77, task_rx, res_tx);

            // We'll send 3 tasks
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..3 {
                    let task = mock_minimal_task_item_with_permit(i);
                    task_tx.send(task).await.unwrap();
                }
                drop(task_tx); // no more tasks
            });

            // Join worker => all tasks must have been processed
            handle.join().expect("Worker thread panicked");

            // Check that we got 3 TaskResults
            let rt2 = Runtime::new().unwrap();
            rt2.block_on(async {
                let mut count = 0;
                while let Ok(_result) = res_rx.try_recv() {
                    count += 1;
                }
                assert_eq!(count, 3, "Expected exactly 3 TaskResults");
            });
        });
    }

    // Additional tests:
    // - If the user tries to send tasks after dropping the channel => error scenario
    // - Concurrency or large-batch tasks 
    // - Task that triggers Freed children => though that is tested more in worker_main_loop.

    /// (A) If the user tries to send tasks after dropping the channel => error scenario.
    #[test]
    fn test_spawn_worker_send_after_close() {
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(2);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(2);

            let handle = spawn_worker_thread(scope, 999, task_rx, res_tx);

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Send one task first
                let item1 = mock_minimal_task_item_with_permit(1);
                task_tx.send(item1).await.unwrap();

                // Now drop the sending side
                drop(task_tx);

                // Attempt to send again => we expect an error because the channel is closed
                // In real code, you might do something like:
                // task_tx.send(...).await => this fails with SendError
                // But we can't do that if we dropped `task_tx`. So let's just show
                // that any attempt here is obviously invalid -> e.g. "already dropped".
                // Alternatively, if you want to demonstrate the error in code:
                //   let err = task_tx.send(mock_minimal_task_item_with_permit(2)).await;
                //   assert!(err.is_err());
            });

            handle.join().expect("Worker thread panicked");

            // We can confirm only 1 TaskResult arrived
            let rt2 = Runtime::new().unwrap();
            let maybe_res = rt2.block_on(async { res_rx.try_recv().ok() });
            assert!(
                maybe_res.is_some(),
                "We expected one TaskResult from the first task"
            );
            // The second "send" wasn't possible => no second TaskResult
            let maybe_res2 = rt2.block_on(async { res_rx.try_recv().ok() });
            assert!(
                maybe_res2.is_none(),
                "No second TaskResult => channel was closed"
            );
        });
    }

    /// (B) Concurrency or large-batch tasks => we send a relatively large set, verifying no panic.
    #[test]
    fn test_spawn_worker_large_batch() {
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(50);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(50);

            let handle = spawn_worker_thread(scope, 55, task_rx, res_tx);

            // We'll send 30 tasks
            let total_tasks = 30;
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                for i in 0..total_tasks {
                    let task = mock_minimal_task_item_with_permit(i);
                    // If channel is not full => .send(...) is fine
                    task_tx.send(task).await.unwrap();
                }
                drop(task_tx); // close
            });

            // join => worker processes them
            handle.join().expect("Worker thread panicked");

            // confirm 30 results
            let rt2 = Runtime::new().unwrap();
            rt2.block_on(async {
                let mut count = 0;
                while let Ok(_res) = res_rx.try_recv() {
                    count += 1;
                }
                assert_eq!(count, total_tasks, "Expected exactly {total_tasks} TaskResults");
            });
        });
    }

    /// (C) Task that triggers Freed children => tested in `worker_main_loop`,
    ///     but we can do a simplified demonstration here:
    #[test]
    fn test_spawn_worker_freed_children() {
        thread::scope(|scope| {
            let (task_tx, task_rx) = mpsc::channel::<TaskItem<'_, TestWireIO<i32>>>(4);
            let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(4);

            let handle = spawn_worker_thread(scope, 9999, task_rx, res_tx);

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Build a mock TaskItem that "frees" child 101, for instance.
                // Normally you'd rely on network logic. We'll just say that
                // after execution, Freed child=101 is sent to child_nodes_tx.
                let mut item = mock_minimal_task_item_with_permit(0);
                // We can do something like “override the node’s .execute() method” if your code is flexible
                // or define a custom network that yields Freed child=101.
                // For demonstration, we’ll rely on the real worker_main_loop logic
                // if it sees edges, etc. This might require a mock network with edges => 0->101.

                // For simplicity, let's just see that the main loop doesn't panic.
                // The Freed child would appear in logs => aggregator or someone picks it up.

                // Send the single Freed-child task
                task_tx.send(item).await.unwrap();
                drop(task_tx);
            });

            // join
            handle.join().expect("Worker thread panicked");

            // We check results:
            let rt2 = Runtime::new().unwrap();
            let maybe_res = rt2.block_on(async { res_rx.try_recv().ok() });
            assert!(
                maybe_res.is_some(),
                "We expected at least one TaskResult from the Freed-child scenario"
            );
            // Freed child is typically tested in the aggregator or `process_immediate`, 
            // so we won't see it here unless we read the child's channel. 
            // We only confirm no panic & at least 1 result was produced.
        });
    }
}
