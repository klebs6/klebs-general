// ---------------- [ File: hydro2-mock/src/mock_network.rs ]
crate::ix!();

pub fn single_noop_operator_i32_network() -> Arc<AsyncMutex<Network::<TestWireIO<i32>>>> 
{

    // Build a "fake" network with 1 node if `node_idx`=0
    // Or build `node_idx+1` nodes if you want each index to be valid.
    let mut net: Network<TestWireIO<i32>> = network!{
        vec![node![0 => NoOpOperator::default()]],
        Vec::new()
    };

    // The network you want for your tests
    let network = Arc::new(AsyncMutex::new(net));

    network
}

pub fn triple_noop_operator_usize_network() -> Arc<AsyncMutex<Network::<TestWireIO<i32>>>> {

    // Build a "fake" network with 1 node if `node_idx`=0
    // Or build `node_idx+1` nodes if you want each index to be valid.
    let mut net = network!{
        vec![
            node![0 => NoOpOperator::default()],
            node![1 => NoOpOperator::default()],
            node![2 => NoOpOperator::default()],
        ],
        Vec::new()
    };

    // The network you want for your tests
    let network = Arc::new(AsyncMutex::new(net));

    network
}

pub fn empty_i32_network() -> Arc<AsyncMutex<Network::<TestWireIO<i32>>>> 
{
    // Build a "fake" network with 1 node if `node_idx`=0
    // Or build `node_idx+1` nodes if you want each index to be valid.
    let mut net = network!{Vec::new(),Vec::new()};

    // The network you want for your tests
    let network = Arc::new(AsyncMutex::new(net));

    network
}

pub fn empty_usize_network() -> Arc<AsyncMutex<Network::<usize>>> 
{
    // Build a "fake" network with 1 node if `node_idx`=0
    // Or build `node_idx+1` nodes if you want each index to be valid.
    let mut net = network!{
        vec![ ],
        Vec::new()
    };

    // The network you want for your tests
    let network = Arc::new(AsyncMutex::new(net));

    network
}

/// Helper function to build a minimal test network with a single node using
/// the `IncrementOperator`.
pub fn build_single_node_network() -> Network<TestWireIO<i32>> {

    network!([ node!(0 => IncrementOperator::default()) ], [])
}

/// Helper to build a multi-node network with each node using `IncrementOperator`.
/// Edges form a chain 0->1->2->3... up to `n - 1`.
pub fn build_chain_network(n: usize, initial_value: i32) -> Network<TestWireIO<i32>> {
    let mut nodes = Vec::with_capacity(n);
    for idx in 0..n {
        let in_buf = if idx == 0 {
            // the first node starts with an initial input
            vec![arc![initial_value]]
        } else {
            // subsequent nodes start empty
            vec![arc![0]]
        };

        let node_built: NetworkNode<TestWireIO<i32>> = node!(idx => IncrementOperator::default());
        nodes.push(node_built);
    }

    // build chain edges
    let mut edges = Vec::with_capacity(n.saturating_sub(1));
    for idx in 0..(n.saturating_sub(1)) {
        // node idx -> node idx+1
        let e = edge!((idx,0) -> ((idx+1),0));
        edges.push(e);
    }

    network!(nodes, edges)
}

/// Helper to build a branching network with concurrency potential.
///        0
///       / \
///      1   2
///       \ /
///        3
pub fn build_branching_network() -> Network<TestWireIO<i32>> {
    network!(
        vec![
            // node 0 => single input=5 => single output
            node!(0 => IncrementOperator::default()),
            // node 1 => single input, single output
            node!(1 => IncrementOperator::default()),
            // node 2 => single input, single output
            node!(2 => IncrementOperator::default()),
            // node 3 => two inputs => so we must have two outputs if using IncrementOperator
            node!(3 => IncrementOperator::default()),
        ],
        vec![
            edge!(0:0 -> 1:0),
            edge!(0:0 -> 2:0),
            // combine 1->3:0 and 2->3:1
            edge!(1:0 -> 3:0),
            edge!(2:0 -> 3:1),
        ]
    )
}
