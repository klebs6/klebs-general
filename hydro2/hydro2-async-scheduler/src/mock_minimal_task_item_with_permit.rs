// ---------------- [ File: hydro2-async-scheduler/src/mock_minimal_task_item_with_permit.rs ]
crate::ix!();

// A minimal function that sets **all** required fields, 
/// including `shared_in_degs`, `child_nodes_tx`, and `completed_nodes`.
#[cfg(test)]
pub fn mock_minimal_task_item_with_permit(node_idx: usize) 
    -> TaskItem<'static, TestWireIO<i32>> 
{
    let fill = true;
    match fill {
        true  => mock_minimal_task_item_with_permit_and_single_noop_operator_network(node_idx),
        false => mock_minimal_task_item_with_permit_and_empty_network(node_idx),
    }
}

#[cfg(test)]
pub fn mock_minimal_task_item_with_permit_and_empty_network(node_idx: usize) 
    -> TaskItem<'static, TestWireIO<i32>> 
{
    let network = empty_i32_network();
    mock_minimal_task_item_with_permit_from_network(network,node_idx)
}

#[cfg(test)]
pub fn mock_minimal_task_item_with_permit_and_single_noop_operator_network(node_idx: usize) 
    -> TaskItem<'static, TestWireIO<i32>> 
{
    let network = single_noop_operator_i32_network();
    mock_minimal_task_item_with_permit_from_network(network,node_idx)
}

#[cfg(test)]
pub fn mock_minimal_task_item_with_permit_from_network(
    network:  Arc<AsyncMutex<Network::<TestWireIO<i32>>>>, 
    node_idx: usize
) -> TaskItem<'static, TestWireIO<i32>> {
    // A real concurrency permit (or None if acquisition fails)
    let real_permit = mock_permit();

    // Provide real or stub arcs for the fields:
    let shared_in_degs  = Arc::new(AsyncMutex::new(vec![])); // empty for a minimal test
    let completed_nodes = SharedCompletedNodes::new(); // also empty
    let (child_nodes_tx, _unused_child_rx) = mpsc::channel::<usize>(16);
    let (ready_nodes_tx, _unused_ready_rx) = mpsc::channel::<usize>(16);

    // Possibly None if you do not do streaming:
    let output_tx: Option<StreamingOutputSender<TestWireIO<i32>>> = None;
    // Possibly None if you do not do checkpointing:
    let checkpoint_cb: Option<Arc<dyn CheckpointCallback>> = None;

    TaskItemBuilder::default()
        .node_idx(node_idx)
        .permit(real_permit)
        .network(network)
        .shared_in_degs(shared_in_degs)
        .output_tx(output_tx)
        .checkpoint_cb(checkpoint_cb)
        .child_nodes_tx(child_nodes_tx)
        .ready_nodes_tx(ready_nodes_tx)
        .completed_nodes(completed_nodes)
        .build()
        .expect("Failed to build TaskItem with real concurrency permit")
}
