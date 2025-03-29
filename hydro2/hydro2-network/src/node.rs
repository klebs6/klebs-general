// ---------------- [ File: src/node.rs ]
crate::ix!();

/// Represents a single node in the network. Each node has
/// an associated operator and references to its input and output buffers.
///
/// TODO: make sure we cant push operators with the same ID
#[derive(Builder,MutGetters,Setters,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub",set = "pub", get_mut = "pub")]
pub struct NetworkNode<NetworkItem> 
where NetworkItem: Debug + Send + Sync
{
    /// Index of this node in the network.
    index: usize,

    /// The operator that this node executes.
    operator: Arc<dyn OperatorInterface<NetworkItem>>,

    /// All input buffers required by this node.
    inputs:  NetworkNodeIoChannelArray<NetworkItem>,

    /// All output buffers that this node will populate.
    outputs: NetworkNodeIoChannelArray<NetworkItem>,
}

impl<NetworkItem> NetworkNode<NetworkItem> 
where NetworkItem: Debug + Send + Sync
{
    /// Acquire read locks on inputs, then call the operator, storing its
    /// results into a local buffer. Finally, acquire write locks and copy
    /// the data into the real outputs via `finish`.
    pub async fn execute(&self) -> NetResult<()> {

        // 1) Acquire read locks
        let mut read_guards: NetworkNodeIoChannelReadGuardArray<'_, NetworkItem> 
            = [None, None, None, None];

        for i in 0..4 {
            if let Some(arc) = &self.inputs[i] {
                read_guards[i] = Some(arc.read().await);
            }
        }

        // Convert to a [Option<&NetworkItem>; 4]
        let inputs: [Option<&NetworkItem>; 4] = [
            read_guards[0].as_ref().map(|g| &**g),
            read_guards[1].as_ref().map(|g| &**g),
            read_guards[2].as_ref().map(|g| &**g),
            read_guards[3].as_ref().map(|g| &**g),
        ];

        // 2) Prepare a local buffer for operator to fill
        let mut output_buffer: NetworkNodeIoChannelValues<NetworkItem> =
            [None, None, None, None];

        // 3) Call the operator asynchronously, passing references
        self.operator.execute(inputs, &mut output_buffer).await?;

        // 4) Acquire write locks
        let mut write_guards: [Option<tokio::sync::RwLockWriteGuard<'_, NetworkItem>>; 4]
            = [None, None, None, None];

        for i in 0..4 {
            if let Some(arc) = &self.outputs[i] {
                write_guards[i] = Some(arc.write().await);
            }
        }

        // 5) Write results from local buffer into outputs
        Self::finish_execution(write_guards, output_buffer)?;

        Ok(())
    }

    fn finish_execution(mut output: NetworkNodeIoChannelWriteGuardArray<'_,NetworkItem>, mut values: NetworkNodeIoChannelValues<NetworkItem>) -> NetResult<()> 
    where NetworkItem: Debug + Send + Sync
    {
        if let Some(o0) = &mut output[0] { if let Some(v) = values[0].take() { **o0 = v; } }
        if let Some(o1) = &mut output[1] { if let Some(v) = values[1].take() { **o1 = v; } }
        if let Some(o2) = &mut output[2] { if let Some(v) = values[2].take() { **o2 = v; } }
        if let Some(o3) = &mut output[3] { if let Some(v) = values[3].take() { **o3 = v; } }
        Ok(())
    }
}

#[macro_export]
macro_rules! node {
    ($idx:expr => $op:expr) => {
        NetworkNodeBuilder::default()
            .index($idx as usize)
            .operator($op.into_arc_operator())
            .inputs([None, None, None, None])
            .outputs([None, None, None, None])
            .build()
            .unwrap()
    };
}

#[cfg(test)]
mod node_macro_tests {
    use super::*;

    #[test]
    fn test_node_macro_single_noop() {
        // We use node!(0 => NoOpOperator::default())
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => NoOpOperator::default());
        assert_eq!(*n0.index(), 0);
        assert_eq!(n0.operator().name(), "default");
        // By default, node! sets inputs/outputs to [None;4]
        for i in 0..4 {
            assert!(n0.inputs()[i].is_none(), "Expected no inputs yet");
            assert!(n0.outputs()[i].is_none(), "Expected no outputs yet");
        }
    }

    #[test]
    fn test_node_macro_with_custom_operator() {
        // Suppose we have AddOp(7).
        let n7: NetworkNode<TestWireIO<i32>> = node!(7 => AddOp::new(7));
        assert_eq!(*n7.index(), 7);
        assert_eq!(n7.operator().name(), "AddOp(+7)");
        // Still no inputs/outputs at creation time
        for i in 0..4 {
            assert!(n7.inputs()[i].is_none());
            assert!(n7.outputs()[i].is_none());
        }
    }
}
