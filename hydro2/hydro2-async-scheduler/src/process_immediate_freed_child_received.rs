// ---------------- [ File: hydro2-async-scheduler/src/process_immediate_freed_child_received.rs ]
crate::ix!();

pub async fn process_immediate_freed_child_received<'threads, T>(
    maybe_child:      Option<usize>,
    child_nodes_tx:   &Sender<usize>,
    worker_pool:      &WorkerPool<'threads, T>,
    completed_nodes:  &SharedCompletedNodes,
    in_flight:        &mut InFlightCounter,
) -> Result<FreedChildReceivedOperationOutcome, NetworkError>
where
    T: Debug + Send + Sync + 'threads,
{
    match maybe_child {
        Some(child_idx) => {
            eprintln!("process_immediate_freed_child_received => Freed child => child_idx={}", child_idx);

            // Perhaps your design calls reenqueue_freed_child(child_idx, child_nodes_tx).await
            // which sends child_idx back to the "ready" channel or something. 
            // Or you might directly call worker_pool.submit(...) if Freed means "submit now".
            // If you do actually submit, in_flight++:

            reenqueue_freed_child(child_idx, child_nodes_tx).await;
            // If that triggers a new “ready” node => in_flight is incremented in “process_immediate_ready_node_received”
            // or you can increment here if you're directly calling worker_pool.

            Ok(FreedChildReceivedOperationOutcome::ChildRequeued)
        }
        None => {
            eprintln!("process_immediate_freed_child_received => Freed channel closed => no more Freed children");
            Ok(FreedChildReceivedOperationOutcome::ChannelClosed)
        }
    }
}

#[cfg(test)]
mod process_immediate_freed_child_received_tests {
    use super::*;
    use tokio::sync::mpsc;

    #[traced_test]
    async fn test_freed_child_some() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        let (child_tx, _child_rx) = mpsc::channel::<usize>(4);
        let (worker_pool, _rx) = WorkerPool::<i32>::new_test_dummy()?;
        let completed_nodes = SharedCompletedNodes::new();

        // Freed child => 999
        let outcome = process_immediate_freed_child_received(
            Some(999),
            &child_tx,
            &worker_pool,
            &completed_nodes,
            &mut in_flight,
        ).await?;

        // We expect Continue, because the channel is still open
        assert_eq!(outcome, FreedChildReceivedOperationOutcome::ChildRequeued);

        Ok(())
    }

    #[traced_test]
    async fn test_freed_child_none() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        // Freed children channel is closed => None => Break
        let (child_tx, _child_rx) = mpsc::channel::<usize>(4);
        let (worker_pool, _rx) = WorkerPool::<i32>::new_test_dummy()?;
        let completed_nodes = SharedCompletedNodes::new();

        let outcome = process_immediate_freed_child_received(
            None,
            &child_tx,
            &worker_pool,
            &completed_nodes,
            &mut in_flight,
        ).await?;

        assert_eq!(outcome, FreedChildReceivedOperationOutcome::ChannelClosed);
        Ok(())
    }
}
