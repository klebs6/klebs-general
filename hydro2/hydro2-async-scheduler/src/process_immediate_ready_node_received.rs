// ---------------- [ File: src/process_immediate_ready_node_received.rs ]
crate::ix!();

pub async fn process_immediate_ready_node_received<'threads, T>(
    maybe_idx:         Option<usize>,
    concurrency_limit: Arc<Semaphore>,
    network:           &Arc<AsyncMutex<Network<T>>>,
    shared_in_degs:    &Arc<AsyncMutex<Vec<usize>>>,
    output_tx:         &Option<StreamingOutputSender<T>>,
    checkpoint_cb:     &Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:    &Sender<usize>,
    ready_nodes_tx:    &Sender<usize>,
    completed_nodes:   &SharedCompletedNodes,
    worker_pool:       &WorkerPool<'threads, T>,
    in_flight:         &mut InFlightCounter
) -> Result<ReadyNodeReceivedOperationOutcome, NetworkError>
where
    T: Debug + Send + Sync + 'threads,
{
    match maybe_idx {
        Some(node_idx) => {
            eprintln!(
                "process_immediate_ready_node_received => got ready node => node_idx={}",
                node_idx
            );

            // Call your existing function: it returns an error or success
            // We might or might not actually submit a node, 
            // but presumably we do. Let's assume it always does.
            handle_new_ready_node(
                node_idx,
                concurrency_limit,
                network,
                shared_in_degs,
                output_tx,
                checkpoint_cb,
                child_nodes_tx,
                ready_nodes_tx,
                completed_nodes,
                worker_pool,
            )
            .await?;

            // If we truly did `worker_pool.submit(...)`, then we do:
            eprintln!("process_immediate_ready_node_received => handled new node => in_flight++");
            in_flight.increment();

            // Optionally poll results 
            // (the parent function is already calling poll_worker_results in a separate select branch, 
            // but you might want to do a quick poll here)
            // poll_worker_results(worker_pool, completed_nodes, in_flight).await?;

            Ok(ReadyNodeReceivedOperationOutcome::NodeSubmitted)
        },
        None => {
            eprintln!(
                "process_immediate_ready_node_received => ready_nodes_rx closed => no more ready nodes"
            );
            Ok(ReadyNodeReceivedOperationOutcome::ChannelClosed)
        }
    }
}

#[cfg(test)]
mod process_immediate_ready_node_received_tests {
    use super::*;
    use tokio::sync::mpsc;

    #[traced_test]
    async fn test_ready_node_some() -> Result<(), NetworkError> {

        let mut in_flight = InFlightCounter::default();

        // Setup concurrency
        let concurrency    = Arc::new(Semaphore::new(2));
        let network        = Arc::new(AsyncMutex::new(Network::<TestWireIO<i32>>::default()));
        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0]));
        let output_tx:     Option<StreamingOutputSender<TestWireIO<i32>>>  = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let (child_nodes_tx, _child_nodes_rx) = mpsc::channel(8);
        let (ready_nodes_tx, _ready_nodes_rx) = mpsc::channel(8);
        let completed_nodes = SharedCompletedNodes::new();

        // A dummy worker pool
        let (worker_pool, _rx) = WorkerPool::<TestWireIO<i32>>::new_test_dummy()?;

        // We pass `Some(42)` as if ready_nodes_rx gave us node=42
        let outcome = process_immediate_ready_node_received(
            Some(42),
            concurrency,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
            &mut in_flight,
        ).await?;

        assert_eq!(outcome, ReadyNodeReceivedOperationOutcome::NodeSubmitted);

        Ok(())
    }

    #[traced_test]
    async fn test_ready_node_none() -> Result<(), NetworkError> {
        let mut in_flight = InFlightCounter::default();
        // If `None` => that means the channel is closed => expect Break
        let concurrency    = Arc::new(Semaphore::new(1));
        let network        = Arc::new(AsyncMutex::new(Network::<TestWireIO<i32>>::default()));
        let shared_in_degs = Arc::new(AsyncMutex::new(vec![]));
        let output_tx:     Option<StreamingOutputSender<TestWireIO<i32>>>  = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let (child_nodes_tx, _) = mpsc::channel(2);
        let (ready_nodes_tx, _) = mpsc::channel(2);
        let completed_nodes     = SharedCompletedNodes::new();
        let (worker_pool, _rx)  = WorkerPool::<TestWireIO<i32>>::new_test_dummy()?;

        let outcome = process_immediate_ready_node_received(
            None,
            concurrency,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
            &mut in_flight,
        ).await?;

        assert_eq!(outcome, ReadyNodeReceivedOperationOutcome::ChannelClosed);
        Ok(())
    }
}
