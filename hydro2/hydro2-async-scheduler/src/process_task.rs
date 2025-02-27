// ---------------- [ File: hydro2-async-scheduler/src/process_task.rs ]
crate::ix!();

/// Processes the `task` by locking the network, executing the node operator,
/// and computing Freed children. More detailed logs to help ensure we see
/// exact timing and concurrency behavior.
pub async fn process_task<'threads, T>(
    task: &mut TaskItem<'threads, T>,
    worker_id: usize,
) -> (Vec<usize>, Option<NetworkError>)
where
    T: Debug + Send + Sync + 'threads,
{
    let node_start = Instant::now();
    let node_idx   = *task.node_idx();
    eprintln!(
        "worker #{worker_id} => process_task => starting node_idx={} at t={:?} (ms resolution)",
        node_idx,
        node_start.elapsed().as_millis(),
    );

    let (freed_children, error) = {
        // Lock the network to ensure consistent node access
        let mut net_guard = task.network().lock().await;
        let node_count    = net_guard.nodes().len();

        if node_idx >= node_count {
            eprintln!(
                "worker #{worker_id} => process_task => node_idx={} is out-of-bounds (network has {} nodes)",
                node_idx,
                node_count
            );
            (Vec::new(), Some(NetworkError::InvalidNode { node_idx }))
        } else {
            // Actually execute
            execute_node(
                &mut net_guard,
                node_idx,
                &task.output_tx(),
                worker_id,
                node_start,
                task.shared_in_degs(),
            ).await
        }
    };

    eprintln!(
        "worker #{worker_id} => process_task => done node_idx={} => freed_children={:?}, error={:?}",
        node_idx,
        freed_children,
        error
    );

    (freed_children, error)
}

#[cfg(test)]
mod process_task_tests {
    use super::*;

    #[traced_test]
    async fn test_process_task_out_of_bounds() {
        // Suppose the network is empty => no nodes
        let mut t = mock_minimal_task_item_with_permit(5);
        {
            let mut net_guard = t.network().lock().await;
            // net_guard has 0 nodes => out of bounds
        }

        let (freed, err) = process_task(&mut t, 123).await;
        assert!(freed.is_empty());
        match err {
            Some(NetworkError::InvalidNode { node_idx }) => {
                assert_eq!(node_idx, 5)
            }
            _ => panic!("Expected InvalidNode error"),
        }
    }

    #[traced_test]
    async fn test_process_task_ok() {
        // Suppose the network has at least 1 node, node_idx=0 => we expect success
        let mut t = mock_minimal_task_item_with_permit(0);
        {
            // Insert at least 1 node so node_idx=0 is valid
            let mut net_guard = t.network().lock().await;
            net_guard.nodes_mut().push(
                node![0 => NoOpOperator::default()]
            );
        }

        let (freed, err) = process_task(&mut t, 0).await;
        assert!(err.is_none());
        // Freed children depends on your network edges => verify as needed
    }
}
