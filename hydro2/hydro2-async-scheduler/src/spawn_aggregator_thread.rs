// ---------------- [ File: hydro2-async-scheduler/src/spawn_aggregator_thread.rs ]
crate::ix!();

pub fn spawn_aggregator<'threads, T>(
    scope:          &'threads Scope<'threads, '_>, 
    main_tasks_rx:  Receiver<TaskItem<'threads, T>>, 
    worker_senders: Vec<Sender<TaskItem<'threads, T>>>

) -> ScopedJoinHandle<'threads, ()> 
where
    T: Debug + Send + Sync + 'threads,
{
    scope.spawn(move || {
        eprintln!("Aggregator => OS thread spawned => building mini runtime");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(aggregator_thread_behavior(main_tasks_rx, worker_senders));
        eprintln!("Aggregator => aggregator OS thread => after block_on => aggregator fully done");
    })
}

#[cfg(test)]
mod spawn_aggregator_tests {

    use super::*;

    /// We define a small helper function that is generic over `'scope`.
    /// This ensures `'scope` matches the aggregator's `'threads` parameter
    /// without "borrowed data escapes" errors.
    fn test_spawn_aggregator_completes_inner<'scope>(
        scope: &'scope Scope<'scope, '_>
    ) {
        // Use `'scope` for your TaskItem
        let (main_tasks_tx, main_tasks_rx) =
            mpsc::channel::<TaskItem<'scope, i32>>(1);

        let (tx, _rx) =
            mpsc::channel::<TaskItem<'scope, i32>>(1);
        let worker_senders = vec![tx];

        // Actually spawn the aggregator using your existing function
        let aggregator_handle = spawn_aggregator(scope, main_tasks_rx, worker_senders);

        // Optionally send tasks, or do nothing:
        // Here, we do nothing => aggregator sees no tasks => it exits.
        drop(main_tasks_tx);

        // Join aggregator. Must be done inside the same scope closure,
        // because aggregator_handle is a ScopedJoinHandle<'scope, ()>.
        aggregator_handle.join().expect("Aggregator panicked");

        // If we get here, aggregator started and ended with no errors.
        eprintln!("test_spawn_aggregator_completes_inner => aggregator done OK");
    }

    /// The actual test. The aggregator is tested by calling our helper.
    #[traced_test]
    fn test_spawn_aggregator_completes() {
        // We open the standard library's scoped thread block
        std::thread::scope(|scope| {
            // Inside this scope, `'scope` is known. We call our helper,
            // unifying the aggregator `'threads` with `'scope`.
            test_spawn_aggregator_completes_inner(scope);
        });

        eprintln!("test_spawn_aggregator_completes => OK");
    }
}
