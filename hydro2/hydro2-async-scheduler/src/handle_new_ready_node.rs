// ---------------- [ File: src/handle_new_ready_node.rs ]
crate::ix!();

/// Called when `select!` yields a new node index from `ready_nodes_rx`.
/// Builds a `TaskItem` and submits it to the worker pool.
pub(crate) async fn handle_new_ready_node<'threads, T>(
    node_idx:          usize,
    concurrency_limit: Arc<Semaphore>,
    network:           &Arc<AsyncMutex<Network<T>>>,
    shared_in_degs:    &Arc<AsyncMutex<Vec<usize>>>,
    output_tx:         &Option<StreamingOutputSender<T>>,
    checkpoint_cb:     &Option<Arc<dyn CheckpointCallback>>,
    child_nodes_tx:    &tokio::sync::mpsc::Sender<usize>,
    ready_nodes_tx:    &tokio::sync::mpsc::Sender<usize>,
    completed_nodes:   &SharedCompletedNodes,
    worker_pool:       &WorkerPool<'threads, T>,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync + 'threads,
{
    eprintln!("handle_new_ready_node => got ready node={}", node_idx);

    // Attempt concurrency permit
    let permit = concurrency_limit.clone().try_acquire_owned().ok();

    // Build a `TaskItem`
    let maybe_task = TaskItemBuilder::default()
        .node_idx(node_idx)
        .permit(permit)
        .network(Arc::clone(network))
        .shared_in_degs(Arc::clone(shared_in_degs))
        .output_tx(output_tx.clone())
        .checkpoint_cb(checkpoint_cb.clone())
        .child_nodes_tx(child_nodes_tx.clone())
        .ready_nodes_tx(ready_nodes_tx.clone())
        .completed_nodes(completed_nodes.clone())
        .build();

    let task = match maybe_task {
        Ok(t) => t,
        Err(_e) => {
            // Return a user-defined error or a fallback
            eprintln!("handle_new_ready_node => failed to build TaskItem for node={}", node_idx);
            return Err(NetworkError::TaskItemBuildFailure {
                node_index: node_idx,
            });
        }
    };

    eprintln!("handle_new_ready_node => submitting task for node {}", node_idx);
    worker_pool.submit(task).await?;

    Ok(())
}

#[cfg(test)]
mod handle_new_ready_node_tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[traced_test]
    async fn test_handle_new_ready_node_ok() {
        // Single concurrency + normal network => everything should be fine.
        let concurrency_limit = Arc::new(Semaphore::new(1));
        let network = Arc::new(AsyncMutex::new(mock_network_ok()));
        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0, 2, 0, 1]));
        let output_tx: Option<StreamingOutputSender<u32>> = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let child_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let ready_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let completed_nodes = SharedCompletedNodes::new();

        // Keep the receiver in scope => channel stays open
        let (worker_pool, _rx) = WorkerPool::new_test_dummy().unwrap();

        let node_idx = 42_usize;
        let result = handle_new_ready_node(
            node_idx,
            concurrency_limit,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;

        assert!(
            result.is_ok(),
            "Expected handle_new_ready_node to succeed with normal concurrency"
        );
    }

    #[traced_test]
    async fn test_handle_new_ready_node_zero_concurrency() {
        // concurrency_limit = 0 => the permit is always None,
        // but as coded, we allow building a TaskItem with no concurrency permit
        // so it should still succeed (assuming no further checks).
        let concurrency_limit = Arc::new(Semaphore::new(0));
        let network = Arc::new(AsyncMutex::new(mock_network_ok()));
        let shared_in_degs = Arc::new(AsyncMutex::new(vec![0]));
        let output_tx: Option<StreamingOutputSender<u32>> = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let child_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let ready_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let completed_nodes = SharedCompletedNodes::new();

        let (worker_pool, _rx) = WorkerPool::new_test_dummy().unwrap();

        let node_idx = 0_usize; // single node
        let result = handle_new_ready_node(
            node_idx,
            concurrency_limit,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;

        // Unless your code specifically checks concurrency is > 0,
        // this might succeed. If your real code requires concurrency>0,
        // you'd expect an error.
        assert!(result.is_ok(), "No concurrency permit => still built TaskItem");
    }

    #[traced_test]
    async fn test_handle_new_ready_node_multiple_submissions() {

        // Test multiple calls => ensure each submission is fine
        let concurrency_limit = Arc::new(Semaphore::new(2));
        let network           = Arc::new(AsyncMutex::new(mock_network_ok()));
        let shared_in_degs    = Arc::new(AsyncMutex::new(vec![0, 0, 0, 0]));

        let output_tx:     Option<StreamingOutputSender<u32>>  = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let child_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let ready_nodes_tx = tokio::sync::mpsc::channel::<usize>(16).0;
        let completed_nodes = SharedCompletedNodes::new();

        let (worker_pool, _rx) = WorkerPool::new_test_dummy().unwrap();

        // Submit node_idx=0
        let res1 = handle_new_ready_node(
            0,
            Arc::clone(&concurrency_limit),
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;
        assert!(res1.is_ok(), "First submission should succeed");

        // Submit node_idx=1
        let res2 = handle_new_ready_node(
            1,
            Arc::clone(&concurrency_limit),
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;
        assert!(res2.is_ok(), "Second submission should succeed");
    }

    #[traced_test]
    async fn test_handle_new_ready_node_task_item_build_fail() {
        // Suppose some scenario that fails the TaskItemBuilder
        // e.g., invalid node_idx or concurrency check or
        // forced with a special WorkerPool that always fails on submit
        let output_tx:     Option<StreamingOutputSender<u32>> = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let concurrency_limit = Arc::new(Semaphore::new(1));
        let network           = Arc::new(AsyncMutex::new(mock_network_ok()));
        // For example, let shared_in_degs be empty => might or might not cause failure
        let shared_in_degs    = Arc::new(AsyncMutex::new(vec![]));
        let child_nodes_tx    = tokio::sync::mpsc::channel::<usize>(16).0;
        let ready_nodes_tx    = tokio::sync::mpsc::channel::<usize>(16).0;
        let completed_nodes   = SharedCompletedNodes::new();

        // A worker_pool that forces `.submit(...)` error
        let (worker_pool, _fake_rx) = WorkerPool::new_test_dummy_causing_error().unwrap();

        let node_idx = 999_usize;
        let result = handle_new_ready_node(
            node_idx,
            concurrency_limit,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;

        // Because the channel is closed from the start => we get an error
        assert!(result.is_err());
    }

    /// A custom test that ensures a specific error from `.submit(...)`.
    /// For instance, if your code can produce `NetworkError::ResourceExhaustion` or another variant,
    /// we can mock that scenario. 
    #[traced_test]
    async fn test_handle_new_ready_node_submit_returns_custom_error() {
        let output_tx:     Option<StreamingOutputSender<u32>>  = None;
        let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

        let concurrency_limit = Arc::new(Semaphore::new(1));
        let network           = Arc::new(AsyncMutex::new(mock_network_ok()));
        let shared_in_degs    = Arc::new(AsyncMutex::new(vec![0]));
        let child_nodes_tx    = tokio::sync::mpsc::channel::<usize>(16).0;
        let ready_nodes_tx    = tokio::sync::mpsc::channel::<usize>(16).0;
        let completed_nodes   = SharedCompletedNodes::new();

        // A worker pool that returns a `ResourceExhaustion` error from submit
        let (worker_pool, _rx) = WorkerPool::new_test_dummy_resource_exhaustion().unwrap();

        let node_idx = 123;
        let result = handle_new_ready_node(
            node_idx,
            concurrency_limit,
            &network,
            &shared_in_degs,
            &output_tx,
            &checkpoint_cb,
            &child_nodes_tx,
            &ready_nodes_tx,
            &completed_nodes,
            &worker_pool,
        ).await;

        // Ensure it's the custom error
        match result {
            Err(NetworkError::ResourceExhaustion { resource }) => {
                assert_eq!(resource, "WorkerPool Main Tasks Channel");
            },
            _ => panic!("Expected ResourceExhaustion from new_test_dummy_resource_exhaustion()"),
        }
    }

    //===============================================================
    // Helper mocks
    //===============================================================
    fn mock_network_ok() -> Network<u32> {
        let mut net = Network::default();
        // ... add some nodes/edges if needed ...
        net
    }
}
