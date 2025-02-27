// ---------------- [ File: src/handle_freed_children.rs ]
crate::ix!();

/// Sends Freed children to `ready_nodes_tx`. This step can be a source of hangs if
/// the channel is at capacity or closed. We add logs at each step to ensure we see
/// exactly where Freed children get queued.
pub async fn handle_freed_children<'threads, T>(
    task: &mut TaskItem<'threads, T>,
    mut freed_children: Vec<usize>,
    worker_id: usize,
)
where
    T: Debug + Send + Sync + 'threads,
{
    let node_idx = *task.node_idx();
    eprintln!(
        "worker #{worker_id} => handle_freed_children => node_idx={} => about to send Freed children={:?}",
        node_idx,
        freed_children
    );

    for c in freed_children.drain(..) {
        eprintln!("worker #{worker_id} => sending Freed child={} from node_idx={}", c, node_idx);
        let send_res = task.ready_nodes_tx().send(c).await;
        if let Err(e) = send_res {
            eprintln!(
                "worker #{worker_id} => ERROR sending Freed child={} => err={:?}",
                c,
                e
            );
        }
    }

    eprintln!(
        "worker #{worker_id} => handle_freed_children => finished => node_idx={}",
        node_idx
    );
}

#[cfg(test)]
mod handle_freed_children_tests {
    use super::*;

    /// 1) Freed children => [1,2,3], normal capacity => read them
    #[traced_test]
    async fn test_handle_freed_children_basic() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let freed = vec![1,2,3];
        let (tx, mut rx) = mpsc::channel::<usize>(10);
        *t.ready_nodes_tx_mut() = tx;

        handle_freed_children(&mut t, freed, 123).await;

        let mut results = Vec::new();
        while let Ok(c) = rx.try_recv() {
            results.push(c);
        }
        results.sort_unstable();
        assert_eq!(results, vec![1,2,3]);
    }

    /// 2) Freed children => [] => no sends
    #[traced_test]
    async fn test_handle_freed_children_empty() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, mut rx) = mpsc::channel::<usize>(2);
        *t.ready_nodes_tx_mut() = tx;

        handle_freed_children(&mut t, vec![], 999).await;

        assert!(rx.try_recv().is_err(), "No Freed => no sends");
    }

    /// 3) Freed children => single => [42]
    #[traced_test]
    async fn test_handle_freed_children_single() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, mut rx) = mpsc::channel::<usize>(2);
        *t.ready_nodes_tx_mut() = tx;

        handle_freed_children(&mut t, vec![42], 123).await;

        let res = rx.try_recv().ok();
        assert_eq!(res, Some(42));
        assert!(rx.try_recv().is_err());
    }

    /// 4) Freed children => multiple => [10,11,12,13]
    #[traced_test]
    async fn test_handle_freed_children_multiple() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, mut rx) = mpsc::channel::<usize>(4);
        *t.ready_nodes_tx_mut() = tx;

        let freed = vec![10, 11, 12, 13];
        handle_freed_children(&mut t, freed, 99).await;

        let mut results = Vec::new();
        while let Ok(x) = rx.try_recv() {
            results.push(x);
        }
        results.sort_unstable();
        assert_eq!(results, vec![10,11,12,13]);
    }

    //---------------------------------------------------------------------------------
    // Next two tests can block under single-thread if we just do a plain .await.
    // We'll fix it by spawning both producer & consumer so they run concurrently.
    //---------------------------------------------------------------------------------

    /// 5) Freed children => exceed capacity => spawn both tasks for concurrency.
    /// If the runtime is single-thread, we ensure no deadlock by letting them
    /// run in parallel with tokio::spawn + join.
    ///
    /// Alternatively, you can do #[tokio::test(flavor="multi_thread")]
    /// to use multiple worker threads. But here we show the spawn approach.
    #[traced_test]
    async fn test_handle_freed_children_exceed_capacity() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, rx) = mpsc::channel::<usize>(2); // capacity=2
        *t.ready_nodes_tx_mut() = tx;

        let freed = vec![100, 101, 102, 103, 104];

        // Producer: handle_freed_children
        let producer = tokio::spawn(async move {
            handle_freed_children(&mut t, freed, 999).await;
        });

        // Consumer: read everything
        let consumer = tokio::spawn(async move {
            let mut out = Vec::new();
            let mut rx = rx; // we own rx now
            while let Some(idx) = rx.recv().await {
                out.push(idx);
            }
            out
        });

        let (prod_res, cons_res) = tokio::join!(producer, consumer);
        prod_res.expect("Producer panicked");
        let results = cons_res.expect("Consumer panicked");

        assert_eq!(results.len(), 5);
        let mut sorted = results.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![100,101,102,103,104]);
    }

    /// 6) Freed children => channel closed => logs, no panic
    #[traced_test]
    async fn test_handle_freed_children_channel_closed() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, rx) = mpsc::channel::<usize>(4);
        drop(rx); // immediately close

        *t.ready_nodes_tx_mut() = tx;

        handle_freed_children(&mut t, vec![50,51], 101).await;
        // No panic => just logs
    }

    /// 7) Freed children => partial read => also spawn both tasks.
    /// The consumer reads 2 items => waits => reads rest => prevents deadlock.
    #[traced_test]
    async fn test_handle_freed_children_partial_read() {
        let mut t = mock_minimal_task_item_with_permit(0);

        let (tx, rx) = mpsc::channel::<usize>(2);
        *t.ready_nodes_tx_mut() = tx;

        let freed = vec![1,2,3,4];

        // Producer
        let producer = tokio::spawn(async move {
            handle_freed_children(&mut t, freed, 55).await;
        });

        // Consumer
        let consumer = tokio::spawn(async move {
            let mut out = Vec::new();
            let mut rx = rx;
            // read first 2
            for _ in 0..2 {
                if let Some(x) = rx.recv().await {
                    out.push(x);
                }
            }
            // small delay => let producer block on 3rd send
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // read the rest
            while let Some(x) = rx.recv().await {
                out.push(x);
            }
            out
        });

        let (p, c) = tokio::join!(producer, consumer);
        p.expect("Producer panicked");
        let results = c.expect("Consumer panicked");
        assert_eq!(results.len(), 4);
        let mut sorted = results.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, vec![1,2,3,4]);
    }

    /// 8) Freed children => random order => confirm exactly that order is sent.
    /// We'll also spawn both tasks to avoid blocking in single-thread.
    #[traced_test]
    async fn test_handle_freed_children_random_order() {
        let mut t = mock_minimal_task_item_with_permit(0);
        let (tx, rx) = mpsc::channel::<usize>(10);
        *t.ready_nodes_tx_mut() = tx;

        let freed = vec![10,7,99,3,42];

        // Producer
        let producer = tokio::spawn(async move {
            handle_freed_children(&mut t, freed.clone(), 777).await;
        });

        // Consumer
        let consumer = tokio::spawn(async move {
            let mut out = Vec::new();
            let mut rx = rx;
            while let Some(x) = rx.recv().await {
                out.push(x);
            }
            out
        });

        let (p, c) = tokio::join!(producer, consumer);
        p.expect("Producer panicked");
        let results = c.expect("Consumer panicked");

        // Freed are sent in the same order .drain() sees them => [10,7,99,3,42]
        assert_eq!(results, vec![10,7,99,3,42]);
    }
}
