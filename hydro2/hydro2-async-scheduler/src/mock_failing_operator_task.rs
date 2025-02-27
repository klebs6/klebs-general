// ---------------- [ File: hydro2-async-scheduler/src/mock_failing_operator_task.rs ]
crate::ix!();

/// Creates a `TaskItem` that uses a `FailingOperator` at `node_idx`.
/// The `reason` is used for the failing operator’s error message.
///
/// The network is built with at least `node_idx+1` nodes, though typically
/// we only need 1 node: the 0-th. Or you can store it exactly at index=0
/// if you like. We place the failing operator at that node. 
pub fn mock_failing_operator_task(node_idx: usize, reason: &str) -> TaskItem<'static, TestWireIO<i32>> {

    // Build a minimal network with 1 node. 
    // We replace that node with FailingOperator.
    let mut net = Network::<TestWireIO<i32>>::default();
    net.nodes_mut().push(
        node![0 => FailingOperator::new("failing_op", reason)]
    );

    // If you want node_idx > 0, you might do a loop to fill up to that index:
    // for i in 0..=node_idx {
    //     net.nodes_mut().push( node![i => if i==node_idx { FailingOperator... } else { NoOpOperator::default() } ] );
    // }

    // concurrency permit
    let real_permit = mock_permit();

    let network = Arc::new(AsyncMutex::new(net));
    let shared_in_degs  = Arc::new(AsyncMutex::new(vec![]));
    let completed_nodes = SharedCompletedNodes::new();
    let (child_nodes_tx, _unused_child_rx) = mpsc::channel::<usize>(16);
    let (ready_nodes_tx, _unused_ready_rx) = mpsc::channel::<usize>(16);

    TaskItemBuilder::default()
        .node_idx(node_idx)
        .permit(real_permit)
        .network(network)
        .shared_in_degs(shared_in_degs)
        .output_tx(None)
        .checkpoint_cb(None)
        .child_nodes_tx(child_nodes_tx)
        .ready_nodes_tx(ready_nodes_tx)
        .completed_nodes(completed_nodes)
        .build()
        .expect("Failed to build failing-operator TaskItem")
}
