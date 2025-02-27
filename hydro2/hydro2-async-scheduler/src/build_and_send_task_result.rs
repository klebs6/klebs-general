// ---------------- [ File: hydro2-async-scheduler/src/build_and_send_task_result.rs ]
crate::ix!();

/// Builds and sends a `TaskResult` to the aggregator (or whomever is listening).
/// Added logs detail the error state and the exact moment we send the result.
pub async fn build_and_send_task_result<'threads, T>(
    task:       TaskItem<'threads, T>,
    error:      Option<NetworkError>,
    results_tx: &Sender<TaskResult>,
    worker_id:  usize,
)
where
    T: Debug + Send + Sync + 'threads
{
    eprintln!(
        "worker #{worker_id} => build_and_send_task_result => preparing TaskResult => node_idx={}, error={:?}",
        task.node_idx(),
        error
    );

    let tres = TaskResultBuilder::default()
        .node_idx(*task.node_idx())
        .freed_children(Vec::new())
        .error(error)
        .build()
        .unwrap();

    eprintln!(
        "worker #{worker_id} => sending TaskResult => {:?}",
        tres
    );

    let _ = results_tx.send(tres).await;

    eprintln!("worker #{worker_id} => build_and_send_task_result => sent");
}

#[cfg(test)]
mod build_and_send_result_tests {
    use super::*;
    use tokio::sync::mpsc;
    use std::sync::Arc;

    /// 1) Basic scenario => error=None => single call => single TaskResult => Freed children=empty
    #[traced_test]
    async fn test_build_and_send_result_ok() {
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        // minimal TaskItem => node_idx=42
        let t = mock_minimal_task_item_with_permit(42);

        // no error => build + send
        build_and_send_task_result(t, None, &res_tx, 999).await;

        // read from res_rx => expect exactly one
        let maybe_tres = res_rx.try_recv().ok();
        assert!(maybe_tres.is_some(), "Expected one TaskResult");
        let tres = maybe_tres.unwrap();

        // confirm node_idx=42, Freed=empty, error=None
        assert_eq!(*tres.node_idx(), 42);
        assert!(tres.error().is_none(), "No error => Ok");
        assert!(tres.freed_children().is_empty(), "Should be empty Freed child");
    }

    /// 2) Single error scenario => e.g. InvalidNode => Freed=empty
    #[traced_test]
    async fn test_build_and_send_result_error() {
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        let t = mock_minimal_task_item_with_permit(5);
        let e = Some(NetworkError::InvalidNode { node_idx: 5 });

        build_and_send_task_result(t, e.clone(), &res_tx, 555).await;

        let r = res_rx.try_recv().ok();
        assert!(r.is_some(), "We sent exactly one => must see one");
        let tres = r.unwrap();

        match tres.error() {
            Some(NetworkError::InvalidNode { node_idx }) => {
                assert_eq!(*node_idx, 5);
            }
            _ => panic!("Expected InvalidNode(5)"),
        }
        // Freed=empty
        assert!(tres.freed_children().is_empty());
    }

    /// 3) Multiple sequential tasks => confirm the aggregator side sees each result in order
    #[traced_test]
    async fn test_build_and_send_multiple() {
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(10);

        // We'll send 3 tasks
        for i in 0..3 {
            let task = mock_minimal_task_item_with_permit(i);
            // Suppose no error => or check i=1 => error, etc. for variety
            let e = if i == 1 {
                Some(NetworkError::InvalidNode { node_idx: 1 })
            } else {
                None
            };
            build_and_send_task_result(task, e, &res_tx, 1000).await;
        }

        // Now read them all
        let mut results = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        // We expect exactly 3
        assert_eq!(results.len(), 3);

        // Index=0 => error=None
        // Index=1 => error=InvalidNode(1)
        // Index=2 => error=None
        // Freed=empty each time
        for (idx, tres) in results.into_iter().enumerate() {
            assert_eq!(*tres.node_idx(), idx as usize);
            if idx == 1 {
                match tres.error() {
                    Some(NetworkError::InvalidNode { node_idx }) => {
                        assert_eq!(*node_idx, 1);
                    }
                    _ => panic!("Expected InvalidNode(1) for idx=1"),
                }
            } else {
                assert!(tres.error().is_none());
            }
            assert!(tres.freed_children().is_empty());
        }
    }

    /// 4) Test with a smaller channel capacity => confirm no blocking or panic
    #[traced_test]
    async fn test_build_and_send_small_capacity() {
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        // consumer in background
        let handle = tokio::spawn(async move {
            let mut results = Vec::new();
            while let Some(r) = res_rx.recv().await {
                results.push(r);
            }
            results
        });

        // produce 2 tasks
        let t1 = mock_minimal_task_item_with_permit(10);
        let t2 = mock_minimal_task_item_with_permit(11);

        build_and_send_task_result(t1, None, &res_tx, 999).await;
        build_and_send_task_result(t2, None, &res_tx, 999).await;

        drop(res_tx);

        let results = handle.await.unwrap();
        assert_eq!(results.len(), 2);
        // Freed=empty each time, error=None each time
        let mut nodes: Vec<usize> = results.iter().map(|x| *x.node_idx()).collect();
        nodes.sort_unstable();
        assert_eq!(nodes, vec![10, 11]);
    }

    /// 5) Distinct error variants => e.g. OperatorFailed vs. InvalidNode
    #[traced_test]
    async fn test_distinct_errors() {
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(5);

        // Build 2 tasks => different errors
        let t_fail_op   = mock_minimal_task_item_with_permit(100);
        let t_invalidnd = mock_minimal_task_item_with_permit(101);

        // We'll build & send both:
        build_and_send_task_result(t_fail_op, 
            Some(NetworkError::OperatorFailed { reason: "op failed".into() }),
            &res_tx,
            123,
        ).await;

        build_and_send_task_result(t_invalidnd, 
            Some(NetworkError::InvalidNode { node_idx: 101}),
            &res_tx,
            123,
        ).await;

        drop(res_tx);

        // read back
        let mut results = Vec::new();
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        assert_eq!(results.len(), 2);

        // check each
        for r in results {
            match r.error() {
                Some(NetworkError::OperatorFailed { reason: msg }) => {
                    assert_eq!(msg, "op failed");
                }
                Some(NetworkError::InvalidNode { node_idx }) => {
                    assert_eq!(*node_idx, 101);
                }
                _ => panic!("Expected either OperatorFailed or InvalidNode(101)"),
            }
            assert!(r.freed_children().is_empty());
        }
    }

    /// 6) Freed-children is always empty => if we later add Freed child logic, we can test it
    #[traced_test]
    async fn test_freed_children_always_empty_for_now() {
        // Just ensure Freed= empty, because the function sets Freed=Vec::new()
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(1);

        let t = mock_minimal_task_item_with_permit(99);
        build_and_send_task_result(t, None, &res_tx, 50).await;

        let tres = res_rx.try_recv().unwrap();
        assert!(tres.freed_children().is_empty(), "Should be empty Freed children");
    }

    /// 7) (Optional) Integration scenario => we can do a small aggregator check,
    ///    but "build_and_send_task_result" is self-contained => we mostly confirm
    ///    it doesn't block or panic with multiple tasks, errors, etc.
    #[traced_test]
    async fn test_integration_scenario() {
        // For demonstration: send multiple tasks, each a random error or none.
        // Then read them in aggregator logic or from res_rx. 
        let (res_tx, mut res_rx) = mpsc::channel::<TaskResult>(10);

        let tasks = vec![
            (10, None),
            (11, Some(NetworkError::OperatorFailed { reason: "fail-op".into() })),
            (12, Some(NetworkError::InvalidNode { node_idx: 12 })),
            (13, None),
        ];

        // build & send
        for (nid, e) in tasks {
            let t = mock_minimal_task_item_with_permit(nid);
            build_and_send_task_result(t, e, &res_tx, 111).await;
        }
        drop(res_tx);

        // read them all
        let mut results = vec![];
        while let Ok(r) = res_rx.try_recv() {
            results.push(r);
        }
        assert_eq!(results.len(), 4);

        // Possibly do a sophisticated check to match node_idx => error type
        // ...
    }
}
