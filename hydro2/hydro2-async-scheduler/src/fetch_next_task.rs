// ---------------- [ File: hydro2-async-scheduler/src/fetch_next_task.rs ]
crate::ix!();

/// Retrieves the next `TaskItem` from the worker’s channel.
/// We now log more details about whether we're waiting, whether a task is present, etc.
/// This can help confirm if a worker is truly idle or if a hang might be from unreceived tasks.
pub async fn fetch_next_task<'threads, T>(
    worker_rx: &mut Receiver<TaskItem<'threads, T>>,
    worker_id: usize,
) -> Option<TaskItem<'threads, T>>
where
    T: Debug + Send + Sync + 'threads
{
    eprintln!("worker #{worker_id} => waiting on fetch_next_task(...)");
    let maybe_task = worker_rx.recv().await;
    match &maybe_task {
        Some(t) => {
            eprintln!(
                "worker #{worker_id} => fetch_next_task => received TaskItem for node={}, concurrency={}",
                t.node_idx(),
                if t.permit().is_some() { "acquired" } else { "none" },
            );
        }
        None => {
            eprintln!("worker #{worker_id} => fetch_next_task => channel closed (no more tasks)");
        }
    }
    maybe_task
}

#[cfg(test)]
mod fetch_next_task_tests {
    use super::*;

    /// 1) If the channel is closed immediately, we expect `None`.
    #[traced_test]
    async fn test_fetch_next_task_empty_channel() {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(1);
        drop(tx); // close immediately

        let maybe_task = fetch_next_task(&mut rx, 0).await;
        assert!(maybe_task.is_none(), "No tasks => None");
    }

    /// 2) Single item => we fetch exactly one => then channel closes => no more.
    #[traced_test]
    async fn test_fetch_next_task_one_item() {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(1);

        let item = mock_minimal_task_item_with_permit(0); 
        tx.send(item).await.unwrap();
        drop(tx); // channel closed after sending one

        let maybe_task = fetch_next_task(&mut rx, 99).await;
        assert!(maybe_task.is_some(), "Expected one task => Some");
        let t = maybe_task.unwrap();
        assert_eq!(*t.node_idx(), 0);

        // Next call => None
        let second = fetch_next_task(&mut rx, 99).await;
        assert!(second.is_none());
    }

    /// 3) Multiple items => we fetch them sequentially with repeated calls.
    #[traced_test]
    async fn test_fetch_next_task_multiple_items() {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(10);

        // Enqueue 3 tasks
        for i in 0..3 {
            let it = mock_minimal_task_item_with_permit(i);
            tx.send(it).await.unwrap();
        }
        drop(tx);

        // fetch #1
        let task1 = fetch_next_task(&mut rx, 1).await;
        assert!(task1.is_some());
        assert_eq!(*task1.as_ref().unwrap().node_idx(), 0);

        // fetch #2
        let task2 = fetch_next_task(&mut rx, 1).await;
        assert!(task2.is_some());
        assert_eq!(*task2.as_ref().unwrap().node_idx(), 1);

        // fetch #3
        let task3 = fetch_next_task(&mut rx, 1).await;
        assert!(task3.is_some());
        assert_eq!(*task3.as_ref().unwrap().node_idx(), 2);

        // done => None
        let t4 = fetch_next_task(&mut rx, 1).await;
        assert!(t4.is_none());
    }

    /// 4) If the channel is unbounded or large capacity, we can do concurrency.
    /// Here we'll spawn a producer that sends tasks slowly, 
    /// and repeatedly call fetch_next_task from the consumer side.
    #[traced_test]
    async fn test_fetch_next_task_with_producer() -> Result<(),NetworkError> {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static, TestWireIO<i32>>>(2);

        // We'll spawn a producer that sends 5 tasks with small delay
        let producer = tokio::spawn(async move {
            for i in 0..5 {
                let item = mock_minimal_task_item_with_permit(i);
                tx.send(item).await.unwrap();
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            // done
        });

        // Meanwhile, we keep calling fetch_next_task in a loop
        let consumer = tokio::spawn(async move {
            let mut out = Vec::new();
            loop {
                match fetch_next_task(&mut rx, 10).await {
                    Some(t) => {
                        out.push(*t.node_idx());
                    }
                    None => break,
                }
                // we can do a small sleep or not
            }
            out
        });

        let (p, results) = tokio::join!(producer, consumer);
        let mut results = results.expect("expected to be able to join");
        p.expect("Producer panicked");
        results.sort_unstable();
        // We expect [0,1,2,3,4]
        assert_eq!(results, vec![0,1,2,3,4]);
        Ok(())
    }

    /// 5) If concurrency_permit is Some or None => we check the logs or
    /// the debug statement "concurrency=acquired" vs "concurrency=none".
    /// We'll just confirm it doesn't break logic, but we can store a "with concurrency" item
    /// or "no concurrency" item and see if we still get them.
    #[traced_test]
    async fn test_fetch_next_task_concurrency_permit_vs_none() {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(2);

        // Item #1 => with concurrency permit
        let mut it1 = mock_minimal_task_item_with_permit(111);
        assert!(it1.permit().is_some()); // by default

        // Item #2 => we forcibly remove the .permit => None
        let mut it2 = mock_minimal_task_item_with_permit(222);
        *it2.permit_mut() = None;

        tx.send(it1).await.unwrap();
        tx.send(it2).await.unwrap();
        drop(tx);

        let first = fetch_next_task(&mut rx, 77).await.unwrap();
        assert_eq!(*first.node_idx(), 111);
        assert!(first.permit().is_some());

        let second = fetch_next_task(&mut rx, 77).await.unwrap();
        assert_eq!(*second.node_idx(), 222);
        assert!(second.permit().is_none());

        let none = fetch_next_task(&mut rx, 77).await;
        assert!(none.is_none());
    }

    /// 6) If we never close the channel, fetch_next_task(...) can block forever 
    /// if there's no tasks. Let's confirm we can do partial concurrency:
    /// We'll do a small test that times out if no items arrive.
    #[traced_test]
    async fn test_fetch_next_task_blocking_scenario() {
        let (tx, mut rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(2);

        // We'll spawn a consumer that does fetch_next_task => we expect it to block 
        // until a task arrives or channel closes.
        // We'll produce 1 task after 100ms.
        let consumer = tokio::spawn(async move {
            let maybe_t = fetch_next_task(&mut rx, 55).await; 
            maybe_t
        });

        let producer = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let item = mock_minimal_task_item_with_permit(999);
            tx.send(item).await.unwrap();
            // do not close channel => let's see if consumer finishes once it gets one.
        });

        let (cons_res, _) = tokio::join!(consumer, producer);
        let maybe_task = cons_res.expect("Consumer panicked");
        assert!(maybe_task.is_some());
        assert_eq!(*maybe_task.as_ref().unwrap().node_idx(), 999);
    }

    /// 7) Large concurrency => multiple fetchers, multiple producers => 
    /// we confirm tasks are distributed among them. This is more aggregator-like logic.
    /// We'll do each fetcher in a separate future, the producers in separate futures, 
    /// and see how many tasks each fetcher obtains.
    #[traced_test]
    async fn test_fetch_next_task_multi_fetchers_producers() {
        let (tx, rx) = mpsc::channel::<TaskItem<'static,TestWireIO<i32>>>(5);
        let rx_arc = Arc::new(AsyncMutex::new(rx));

        let txc1 = tx.clone();
        let txc2 = tx.clone();

        // We'll spawn 2 producers => each sends 5 tasks => total=10.
        let p1 = {
            tokio::spawn(async move {
                for i in 0..5 {
                    let item = mock_minimal_task_item_with_permit(i);
                    txc1.send(item).await.unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            })
        };

        let p2 = {
            tokio::spawn(async move {
                for j in 5..10 {
                    let item = mock_minimal_task_item_with_permit(j);
                    // the original tx, since we didn't drop it
                    // or we do a clone again
                    // but let's do a new clone:
                    txc2.send(item).await.unwrap();
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                }
            })
        };

        // We'll spawn 2 fetchers => each tries to read tasks in a loop until channel is closed
        // but we don't plan to close the channel => let's do a short approach:
        // Actually let's forcibly close after producers are done => or we can't get `None`.
        let rxa = rx_arc.clone();
        let fetcher_a = tokio::spawn(async move {
            let mut out = Vec::new();
            loop {
                let mut guard = rxa.lock().await;
                if let Some(t) = fetch_next_task(&mut *guard, 111).await {
                    out.push(*t.node_idx());
                } else {
                    break;
                }
            }
            out
        });

        let rxb = rx_arc.clone();
        let fetcher_b = tokio::spawn(async move {
            let mut out = Vec::new();
            loop {
                let mut guard = rxb.lock().await;
                if let Some(t) = fetch_next_task(&mut *guard, 222).await {
                    out.push(*t.node_idx());
                } else {
                    break;
                }
            }
            out
        });

        // Wait for producers to finish => then drop the main tx => that ensures channel yields None
        let _ = tokio::join!(p1, p2);
        drop(tx); // channel is closed => subsequent fetch_next_task => None

        let (a_res, b_res) = tokio::join!(fetcher_a, fetcher_b);

        let list_a = a_res.expect("FetcherA panicked");
        let list_b = b_res.expect("FetcherB panicked");
        let total_count = list_a.len() + list_b.len();

        // total tasks => 10 (5 from p1, 5 from p2)
        assert_eq!(total_count, 10);

        // we can also check there's no duplication => combine + sort + unique
        let mut combined = Vec::new();
        combined.extend(list_a);
        combined.extend(list_b);
        combined.sort_unstable();
        combined.dedup();
        assert_eq!(combined, vec![0,1,2,3,4,5,6,7,8,9]);
    }
}
