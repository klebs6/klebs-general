// ---------------- [ File: src/reenqueue_freed_child.rs ]
crate::ix!();

/// Called when we get a freed child index from `child_nodes_rx`.
/// We try to push it into `child_nodes_tx` for re-processing.  
pub(crate) async fn reenqueue_freed_child(
    child_idx: usize,
    child_nodes_tx: &tokio::sync::mpsc::Sender<usize>,
) {
    eprintln!("process_immediate => Freed child={}", child_idx);
    if let Err(_e) = child_nodes_tx.send(child_idx).await {
        eprintln!(
            "process_immediate => failed to re-enqueue child={}",
            child_idx
        );
    }
}

#[cfg(test)]
mod reenqueue_freed_child_tests {
    use super::*;

    /// 1) Basic single-child reenqueue => we confirm the child arrives in the channel.
    #[traced_test]
    async fn test_reenqueue_freed_child_ok() {
        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(2);
        // Freed child
        let freed_child = 7_usize;

        reenqueue_freed_child(freed_child, &child_tx).await;

        // Check that it arrives
        let next = child_rx.recv().await;
        assert_eq!(next, Some(7), "Expected to receive child=7 in child_rx");
    }

    /// 2) If child_tx is closed, ensure we log but do not panic or throw an error.
    ///    The function just logs and returns.
    #[traced_test]
    async fn test_reenqueue_freed_child_channel_closed() {
        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(2);
        drop(child_rx); // Immediately drop receiver

        reenqueue_freed_child(99, &child_tx).await;
        // No panic => we are good. Possibly capture logs to ensure we see the eprintln.
    }

    /// 4) If the channel is bounded and we exceed its capacity, `send` might fail
    ///    or block. By default, `send(...).await` *blocks* if the channel is full.
    ///    We can test that scenario. If we want to ensure a simple 'log and proceed'
    ///    on error, we can reduce the channel capacity to 1, and do 2 enqueues 
    ///    without draining.
    ///
    ///    **Note**: The default is that `.send(...).await` will block until there's space.
    ///    If you do want to see an immediate "failed to re-enqueue" log, you might switch
    ///    to `try_send(...)` in your real code. Otherwise, to see an error, you must
    ///    drop the receiver or forcibly close the channel. We'll demonstrate 
    ///    a "blocked" scenario with a small timeout.
    #[traced_test]
    async fn test_reenqueue_freed_child_channel_full() {
        use tokio::time::{timeout, Duration};

        // Channel of capacity=1
        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(1);

        // 1) Put something in the channel
        child_tx.try_send(999).expect("Should succeed first time");
        
        // 2) Now the channel is full, reenqueue another child => blocks
        let blocked_future = reenqueue_freed_child(1000, &child_tx);

        // We wrap in a small timeout
        let maybe_res = timeout(Duration::from_millis(200), blocked_future).await;

        match maybe_res {
            Ok(()) => panic!("We expected the future to block, but it completed quickly."),
            Err(_elapsed) => {
                // This means our future didn't complete in 200ms => it's blocked.
                // We can confirm the function does not panic or return an error,
                // but it is stuck. In a real scenario, you'd want an aggregator or
                // a receiver to read from the channel so it unblocks eventually.
            }
        }

        // For demonstration: if we read from child_rx once, the second send can proceed:
        // child_rx.recv().await; // unblocks the second send
    }

    /// 5) Freed child with random values. This test just demonstrates
    ///    that we can reenqueue arbitrary child indices. 
    ///    We confirm they come out in the same order.
    #[tokio::test]
    async fn test_reenqueue_random_values() {
        use rand::{thread_rng, Rng};

        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(10);

        let mut rng = thread_rng();
        let random_values: Vec<usize> = (0..5).map(|_| rng.gen_range(0..1000)).collect();

        for val in &random_values {
            reenqueue_freed_child(*val, &child_tx).await;
        }

        // read them back
        let mut results = Vec::new();
        for _ in 0..random_values.len() {
            if let Some(r) = child_rx.recv().await {
                results.push(r);
            }
        }

        assert_eq!(results, random_values,
            "We should get the same Freed child indices in order");
    }

    /// 3) Multiple Freed children in quick succession. We want to verify
    ///    that each call to reenqueue_freed_child enqueues its own index.
    #[traced_test]
    async fn test_reenqueue_multiple_children() {
        let (child_tx, mut child_rx) = tokio::sync::mpsc::channel::<usize>(5);

        // Freed children: 10, 11, 12, 13
        for i in 10..14 {
            reenqueue_freed_child(i, &child_tx).await;
        }

        // Drop the sender so that the channel eventually closes and `recv()` can return `None`.
        drop(child_tx);

        let mut received = Vec::new();
        while let Some(idx) = child_rx.recv().await {
            received.push(idx);
        }

        // Check that we read [10, 11, 12, 13]
        assert_eq!(
            received,
            vec![10, 11, 12, 13],
            "Expected Freed children 10..13 in that order"
        );
    }
}
