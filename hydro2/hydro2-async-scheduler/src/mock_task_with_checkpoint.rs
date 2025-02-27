// ---------------- [ File: hydro2-async-scheduler/src/mock_task_with_checkpoint.rs ]
crate::ix!();

/// A global list of node indices that were checkpointed, for test verification.
lazy_static! {
    static ref CHECKPOINT_INVOCATIONS: StdMutex<Vec<usize>> = StdMutex::new(Vec::new());
}

#[derive(Debug)]
struct MockCheckpointCallback;

#[async_trait]
impl CheckpointCallback for MockCheckpointCallback {
    async fn checkpoint(&self, completed_nodes: &[usize]) -> Result<(), NetworkError> {
        // We'll just push the *last* node or do something fancier
        if let Some(last_idx) = completed_nodes.last() {
            let mut guard = CHECKPOINT_INVOCATIONS.lock().unwrap();
            guard.push(*last_idx);
        }
        Ok(())
    }
}

/// Returns the global list of checkpointed nodes, for assertion in tests.
pub fn get_mock_checkpoint_invocations() -> Vec<usize> {
    CHECKPOINT_INVOCATIONS.lock().unwrap().clone()
}

/// Clears the global list, so each test can start fresh.
pub fn clear_checkpoint_invocations() {
    CHECKPOINT_INVOCATIONS.lock().unwrap().clear();
}

pub fn mock_task_with_checkpoint<'a>(node_idx: usize) -> TaskItem<'a, TestWireIO<i32>> {
    // minimal net
    let mut net = Network::<TestWireIO<i32>>::default();
    net.nodes_mut().push( node![0 => NoOpOperator::default()] );

    let real_permit = mock_permit();

    let network         = Arc::new(AsyncMutex::new(net));
    let shared_in_degs  = Arc::new(AsyncMutex::new(vec![]));
    let completed_nodes = SharedCompletedNodes::new();
    let (child_nodes_tx, _child_rx) = mpsc::channel::<usize>(16);
    let (ready_nodes_tx, _ready_rx) = mpsc::channel::<usize>(16);

    // The custom checkpoint callback
    let callback = Arc::new(MockCheckpointCallback);

    TaskItemBuilder::default()
        .node_idx(node_idx)
        .permit(real_permit)
        .network(network)
        .shared_in_degs(shared_in_degs)
        .output_tx(None)
        .checkpoint_cb(Some(callback as Arc<dyn CheckpointCallback>))
        .child_nodes_tx(child_nodes_tx)
        .ready_nodes_tx(ready_nodes_tx)
        .completed_nodes(completed_nodes)
        .build()
        .unwrap()
}
