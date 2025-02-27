// ---------------- [ File: hydro2-network/src/network.rs ]
crate::ix!();

/// The full network containing nodes, edges, and any relevant metadata.
#[derive(Default,Builder,MutGetters,Setters,Getters,Debug,Clone)]
#[getset(get="pub",set = "pub", get_mut = "pub")]
#[builder(setter(into))]
pub struct Network<NetworkItem> 
where NetworkItem: Debug + Send + Sync
{
    /// All nodes present in this network.
    nodes: Vec<NetworkNode<NetworkItem>>,
    /// The directed edges forming the DAG between nodes.
    edges: Vec<NetworkEdge>,
}

impl<NetworkItem> Network<NetworkItem> 
where NetworkItem: Debug + Send + Sync
{
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}

#[macro_export]
macro_rules! network {
    ($nodes:expr, $edges:expr) => {{
        let mut net = NetworkBuilder::default()
            .nodes($nodes)
            .edges($edges)
            .build()
            .unwrap();

        // 4) Possibly do net.validate() => checks cycles, etc.
        if let Err(e) = net.validate() {
            panic!("network validation saw an error: {:#?}", e);
        }

        // 5) Wire up => automatically allocate arcs for each node’s outputs & link them
        if let Err(e) = wire_up_network(&mut net) {
            panic!("network wiring saw an error: {:#?}", e);
        }

        net
    }};
}

#[cfg(test)]
mod network_macro_tests {
    use super::*;

    #[test]
    fn test_network_macro_basic() -> Result<(), NetworkError> {
        // We'll define 2 nodes, 1 edge => build a small network
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(100));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(42));
        let e0 = edge!(0:0 -> 1:0);

        // Use the macro
        let net = network!(vec![n0, n1], vec![e0]);
        // That calls net.validate() + wire_up_network(...) under the hood.

        // Now net is fully wired
        // Node0 => input_count=0 => output_count=1 => 
        // Node1 => input_count=1 => output_count=1
        // Edge => 0:0 -> 1:0 means node1 inputs[0] is the same Arc as node0 outputs[0]
        assert_eq!(net.nodes().len(), 2);
        assert_eq!(net.edges().len(), 1);

        // Node0 => outputs[0] must be Some(...)
        assert!(net.nodes()[0].outputs()[0].is_some());
        // Node1 => inputs[0] must be Some
        assert!(net.nodes()[1].inputs()[0].is_some());

        Ok(())
    }

    #[test]
    fn test_network_macro_fanout() -> Result<(), NetworkError> {
        let cst: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(10));
        let aop: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(5));
        let mop: NetworkNode<TestWireIO<i32>> = node!(2 => MultiplyOp::new(2));
        let net = network!(
            vec![cst, aop, mop],
            vec![
                edge!(0:0 -> 1:0),
                edge!(0:0 -> 2:0),
            ]
        );
        // Should be valid, Node1 and Node2 each take the same output from Node0
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Cycle detected")]
    fn test_network_macro_cycle_panics() {
        // if we introduce a cycle => net.validate() should fail => triggers panic in the macro
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => NoOpOperator::default());
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => NoOpOperator::default());
        // cycle
        let _ = network!(
            vec![n0, n1],
            vec![
                edge!(0:0 -> 1:0),
                edge!(1:0 -> 0:0),
            ]
        );
    }
}
