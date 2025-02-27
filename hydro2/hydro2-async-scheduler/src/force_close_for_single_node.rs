// ---------------- [ File: hydro2-async-scheduler/src/force_close_for_single_node.rs ]
crate::ix!();

/// Forcefully closes channels + aggregator for the single-node case:
/// - Drops `child_nodes_tx`,
/// - Drops `ready_nodes_tx`,
/// - Calls `worker_pool.close_main_tasks_channel()`.
///
/// This is a pure "close everything right now" step. It does not enqueue anything,
/// nor does it rely on `initialize_zero_degree_nodes`. It's an explicit action:
/// 
/// ```ignore
/// if node_count == 1 {
///     force_close_for_single_node(&ready_nodes_tx, &child_nodes_tx, &worker_pool);
/// }
/// ```
/// 
/// After calling, no further items can be sent on `ready_nodes_tx` or `child_nodes_tx`,
/// and aggregator sees a `None` so it stops. This typically finalizes scheduling
/// for the single-node scenario.
pub(crate) fn force_close_for_single_node<'threads, T>(
    ready_nodes_tx: tokio::sync::mpsc::Sender<usize>,
    child_nodes_tx: tokio::sync::mpsc::Sender<usize>,
    worker_pool:    &WorkerPool<'threads, T>,
)
where
    T: std::fmt::Debug + Send + Sync + 'threads
{
    // Freed child channel
    drop(child_nodes_tx);
    eprintln!("force_close_for_single_node => Freed children channel closed");

    // Freed ready channel
    drop(ready_nodes_tx);
    eprintln!("force_close_for_single_node => Ready nodes channel closed");

    // aggregator
    worker_pool.close_main_tasks_channel();
    eprintln!("force_close_for_single_node => forcibly closed Freed + aggregator");
}

#[cfg(test)]
mod force_close_for_single_node_tests {
    use super::*;

    /// 1) Basic test: ensure `force_close_for_single_node` drops both `ready` and `child`
    /// channels, and calls `worker_pool.close_main_tasks_channel()`.
    #[traced_test]
    async fn test_force_close_for_single_node_basic() -> Result<(), NetworkError> {
        eprintln!("\n=== test_force_close_for_single_node_basic ===");

        // (1) Create channels, plus a mock WorkerPool
        let (ready_tx, ready_rx) = channel::<usize>(4);
        let (child_tx, child_rx) = channel::<usize>(4);

        let (pool_raw, _dummy_rx) = WorkerPool::<usize>::new_test_dummy()?;
        let worker_pool = Arc::new(pool_raw);

        // (2) We do NOT attempt to send anything before the call.
        //     We just call force_close_for_single_node immediately:
        force_close_for_single_node(ready_tx, child_tx, worker_pool.as_ref());
        // at this point, those two Sender<usize> objects have been moved (and dropped).

        // (3) Because the senders are dropped, the channel from the aggregator's perspective
        //     is closed. We can confirm from the receiver side. Let’s see if we get `None`.
        let rread = drain_channel(ready_rx).await;
        let cread = drain_channel(child_rx).await;
        assert!(rread.is_empty(), "No items => ready channel was closed immediately");
        assert!(cread.is_empty(), "No items => child channel was closed immediately");

        // (4) Aggregator: final shutdown is a no-op in test. We must unwrap the Arc if we want 
        //     to consume the WorkerPool by-value.
        match Arc::try_unwrap(worker_pool) {
            Ok(pool) => pool.shutdown(),
            Err(_still_arc) => eprintln!("Not the only Arc reference => cannot shutdown fully"),
        }

        Ok(())
    }

    /// 2) If we call `force_close_for_single_node` multiple times,
    /// the second+ calls do nothing extra but also do not panic.
    ///
    /// Because `force_close_for_single_node` takes ownership,
    /// we must decide which “Sender” we pass on the second call—actually it’s gone!
    /// So we demonstrate a scenario: we create new channels each time, or we skip the second call
    /// for the same channel. 
    ///
    /// In practice, calling it multiple times on the *same channels* is meaningless once the first
    /// call has consumed them. So we can show two approach:
    ///   A) We either skip the second call because we no longer have `ready_tx, child_tx`.
    ///   B) We make the second call with already-dropped senders => no effect, no panic.
    #[traced_test]
    async fn test_force_close_for_single_node_multiple_calls() -> Result<(), NetworkError> {
        eprintln!("\n=== test_force_close_for_single_node_multiple_calls ===");

        let (ready_tx, ready_rx) = channel::<usize>(1);
        let (child_tx, child_rx) = channel::<usize>(1);

        let (pool_raw, _rx) = WorkerPool::<usize>::new_test_dummy()?;
        let worker_pool = Arc::new(pool_raw);

        // (A) First call => forcibly closes
        force_close_for_single_node(ready_tx, child_tx, worker_pool.as_ref());

        // We can read from the receivers:
        let ritems = drain_channel(ready_rx).await;
        let citems = drain_channel(child_rx).await;
        assert!(ritems.is_empty(), "First call => aggregator channels closed => no items");
        assert!(citems.is_empty());

        // (B) There's no second call with the SAME channels, because they've been consumed.
        //     But if we do want to demonstrate “multiple times,” we must create new channels:
        let (ready_tx2, ready_rx2) = channel::<usize>(1);
        let (child_tx2, child_rx2) = channel::<usize>(1);

        // Force close again:
        force_close_for_single_node(ready_tx2, child_tx2, worker_pool.as_ref());
        // No panic => does nothing.

        let r2 = drain_channel(ready_rx2).await;
        let c2 = drain_channel(child_rx2).await;
        assert!(r2.is_empty(), "Second call => aggregator channels #2 closed => no items");
        assert!(c2.is_empty());

        match Arc::try_unwrap(worker_pool) {
            Ok(pool) => pool.shutdown(),
            Err(_)   => eprintln!("Arc still held => skipping shutdown"),
        }
        Ok(())
    }

    /// 3) A minimal concurrency demonstration:
    /// one task tries sending, another calls `force_close_for_single_node`.
    /// We ensure no indefinite block by giving the channel a big capacity,
    /// and we drop the `ready_tx_clone` once done sending.
    #[traced_test]
    async fn test_force_close_for_single_node_with_concurrent_sends() -> Result<(), NetworkError> {

        eprintln!("\n=== test_force_close_for_single_node_with_concurrent_sends ===");

        // Use a somewhat larger capacity so we don't block.
        let (ready_tx, ready_rx) = channel::<usize>(16);
        let (child_tx, child_rx) = channel::<usize>(16);

        let (pool_raw, _rx) = WorkerPool::<usize>::new_test_dummy()?;
        let worker_pool = Arc::new(pool_raw);

        // We keep the "main" senders un‐used except to pass them into `force_close_for_single_node`.
        // We'll spawn a sender task that uses a clone:
        let ready_tx_clone = ready_tx.clone();
        let sender_task = tokio::spawn(async move {
            for i in 0..5 {
                // We do not block if capacity=16 and no consumer => 5 << 16
                let _ = ready_tx_clone.send(i).await;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            // After sending is done, drop the clone so the channel can actually close
            drop(ready_tx_clone);
        });

        // Sleep a bit, then forcibly close the aggregator side
        let closer_pool = Arc::clone(&worker_pool);
        let closer_task = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            force_close_for_single_node(ready_tx, child_tx, closer_pool.as_ref());
        });

        // Wait for both tasks
        let _ = futures::future::join(sender_task, closer_task).await;

        // The aggregator side is forcibly closed. The sender clone is also dropped after sending.
        // So from the receiver side:
        let items = drain_channel(ready_rx).await;
        eprintln!("Got items from ready_rx = {:?}", items);

        // child_rx was never used => expect empty
        let citems = drain_channel(child_rx).await;
        assert!(citems.is_empty());

        // aggregator: attempt final aggregator shutdown
        match Arc::try_unwrap(worker_pool) {
            Ok(pool) => pool.shutdown(),
            Err(_)   => eprintln!("Arc still held => skipping shutdown"),
        }
        Ok(())
    }

    /// 4) If we haven't used the channels, `force_close_for_single_node` is trivial.
    /// We do not attempt to do more with them after the function call.
    #[traced_test]
    async fn test_force_close_for_single_node_unused_channels() -> Result<(), NetworkError> {
        eprintln!("\n=== test_force_close_for_single_node_unused_channels ===");

        let (ready_tx, ready_rx) = channel::<usize>(8);
        let (child_tx, child_rx) = channel::<usize>(8);

        let (pool_raw, _rx) = WorkerPool::<usize>::new_test_dummy()?;
        let worker_pool = Arc::new(pool_raw);

        // No usage, so just call force_close:
        force_close_for_single_node(ready_tx, child_tx, worker_pool.as_ref());

        // We cannot do `ready_tx.send(...)` after that line because `ready_tx` is moved.
        // But from the receiver side, let's see if we get anything:
        let final_ready = drain_channel(ready_rx).await;
        assert!(final_ready.is_empty());

        let final_children = drain_channel(child_rx).await;
        assert!(final_children.is_empty());

        // aggregator is also closed => attempt final aggregator shutdown
        match Arc::try_unwrap(worker_pool) {
            Ok(pool) => pool.shutdown(),
            Err(_)   => eprintln!("Arc still held => skipping shutdown"),
        }
        Ok(())
    }
}
