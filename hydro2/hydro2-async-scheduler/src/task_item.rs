// ---------------- [ File: src/task_item.rs ]
crate::ix!();

/// Each node execution is one `TaskItem`.
#[derive(Debug,Builder,MutGetters,Getters)]
#[builder(setter(into),pattern="owned")]
#[getset(get = "pub", get_mut="pub")]
pub struct TaskItem<'threads,T>
where
    T: Debug + Send + Sync + 'threads,
{
    /// Which node index to run
    node_idx:           usize,
    /// The concurrency permit
    permit:             Option<OwnedSemaphorePermit>,
    /// The network
    network:            Arc<AsyncMutex<Network<T>>>,
    /// Shared in-degrees
    shared_in_degs:     Arc<AsyncMutex<Vec<usize>>>,
    /// If streaming is enabled
    output_tx:          Option<StreamingOutputSender<T>>,
    /// Checkpoint callback
    checkpoint_cb:      Option<Arc<dyn CheckpointCallback>>,
    /// Freed children => push to this channel
    child_nodes_tx:     tokio::sync::mpsc::Sender<usize>,
    /// Completed
    completed_nodes:    SharedCompletedNodes,

    /// Freed children => we used to put them in child_nodes_tx, 
    /// but we actually want them to go into ready_nodes_tx
    ready_nodes_tx:     tokio::sync::mpsc::Sender<usize>,

    #[builder(default)]
    threads_lifetime:   std::marker::PhantomData<&'threads ()>,
}

#[macro_export]
macro_rules! task_item {
    (
        node_idx:         $node_idx:expr, 
        permit:           $permit:expr, 
        network:          $network:expr, 
        shared_in_degs:   $shared_in_degs:expr, 
        output_tx:        $output_tx:expr, 
        checkpoint_cb:    $checkpoint_cb:expr, 
        child_nodes_tx:   $child_nodes_tx:expr, 
        ready_nodes_tx:   $ready_nodes_tx:expr, 
        completed_nodes:  $completed_nodes:expr
    ) => {{
        TaskItemBuilder::default()
            .node_idx($node_idx)
            .permit($permit)
            .network(Arc::clone(&$network))
            .shared_in_degs(Arc::clone(&$shared_in_degs))
            .output_tx($output_tx.clone())
            .checkpoint_cb($checkpoint_cb.clone())
            .child_nodes_tx($child_nodes_tx.clone())
            .ready_nodes_tx($ready_nodes_tx.clone())
            .completed_nodes($completed_nodes.clone())
            .build()
            .expect("Failed to build TaskItem")
    }};
}
