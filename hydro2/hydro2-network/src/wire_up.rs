// ---------------- [ File: hydro2-network/src/wire_up.rs ]
crate::ix!();

/// Wires up all outputs and inputs across the network, checking port‐range, type‐string compatibility,
/// required‐connection fulfillment, and fan‐in/fan‐out rules. Returns an error if any violation occurs.
pub fn wire_up_network<NetworkItem>(
    net: &mut Network<NetworkItem>
) -> NetResult<()>
where
    NetworkItem: Debug + Send + Sync + Default,
{
    // 1) Pre-allocate the outputs for each node based on operator.output_count().
    //    We support up to 4 physical outputs. If operator claims >4, fail.
    for (node_idx, node) in net.nodes_mut().iter_mut().enumerate() {
        let out_count = node.operator().output_count();
        if out_count > 4 {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Node #{} operator output_count={} exceeds the max of 4 ports",
                    node_idx, out_count
                ),
            });
        }
        // Allocate (Arc<AsyncRwLock<NetworkItem>>) for each real port:
        for port_idx in 0..out_count {
            node.outputs_mut()[port_idx] = Some(Arc::new(AsyncRwLock::new(NetworkItem::default())));
        }
        // Ensure leftover "physical" slots up to 4 are None:
        for port_idx in out_count..4 {
            node.outputs_mut()[port_idx] = None;
        }
    }

    // 2) For each node, we track how many inputs are actually used in total
    //    (kept for historical exact-match checks) plus per-port usage.
    let mut used_input_count: Vec<usize> = vec![0; net.nodes().len()];
    // Also track usage counts to validate required connections:
    let mut input_usage = vec![[0usize; 4]; net.nodes().len()];   // node_idx -> [port0_count, port1_count, ...]
    let mut output_usage = vec![[0usize; 4]; net.nodes().len()];  // node_idx -> [port0_count, port1_count, ...]

    // We'll clone edges since we might do multiple passes
    let edges = net.edges().clone();

    // 3) Connect each edge => set the downstream node’s input array slot
    //    to the same arc used by the upstream node’s output array slot.
    for (edge_idx, edge) in edges.iter().enumerate() {
        let src = edge.source_index();
        let so  = edge.source_output_idx();
        let dst = edge.dest_index();
        let di  = edge.dest_input_idx();

        // Basic node‐index range checks:
        if *src >= net.nodes().len() || *dst >= net.nodes().len() {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} references invalid node index (src={}, dst={}, node_count={})",
                    edge_idx, src, dst, net.nodes().len()
                ),
            });
        }
        let src_op = &net.nodes()[*src].operator();
        let dst_op = &net.nodes()[*dst].operator();

        // Upstream operator’s output_count => so must be < that count
        if *so >= src_op.output_count() {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} references source node {} output port {}, but operator only has {} outputs",
                    edge_idx, src, so, src_op.output_count()
                ),
            });
        }
        // Downstream operator’s input_count => di must be < that count
        if *di >= dst_op.input_count() {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} references dest node {} input port {}, but operator only has {} inputs",
                    edge_idx, dst, di, dst_op.input_count()
                ),
            });
        }

        // Check physical limit of 4:
        if *so >= 4 || *di >= 4 {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} references port so={}, di={} >= 4, out of range for 4 ports",
                    edge_idx, so, di
                ),
            });
        }

        // 3a) Check type‐string matching: out_str == in_str
        // If either operator reports None for that port, treat as a mismatch.
        let out_str = match src_op.output_port_type_str(*so) {
            Some(s) => s,
            None => {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!(
                        "Edge #{} source node {} output port {} has no declared type string",
                        edge_idx, src, so
                    ),
                });
            }
        };
        let in_str = match dst_op.input_port_type_str(*di) {
            Some(s) => s,
            None => {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!(
                        "Edge #{} dest node {} input port {} has no declared type string",
                        edge_idx, dst, di
                    ),
                });
            }
        };
        if out_str != in_str {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} type mismatch: source node {} output port {} => {:?} != dest node {} input port {} => {:?}",
                    edge_idx, src, so, out_str, dst, di, in_str
                ),
            });
        }

        // 3b) Retrieve Arc from node[src].outputs[so] and assign to node[dst].inputs[di]
        let arc_opt = net.nodes()[*src].outputs()[*so].clone();
        if arc_opt.is_none() {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Edge #{} references source node {} output port {}, but that port is None",
                    edge_idx, src, so
                ),
            });
        }
        net.nodes_mut()[*dst].inputs_mut()[*di] = arc_opt;

        // 3c) Keep track of usage:
        used_input_count[*dst] += 1;
        input_usage[*dst][*di] += 1;
        output_usage[*src][*so] += 1;
    }

    // 4) Post-check: 
    //    a) Each node’s total used inputs must match node.operator().input_count() => historical exact‐match rule
    //    b) Ensure no input port is double‐fed (input_usage>1)
    //    c) Enforce required connections for input ports
    //    d) Enforce required connections for output ports
    for (node_idx, node) in net.nodes().iter().enumerate() {
        let op = node.operator();
        let in_count = op.input_count();
        let out_count = op.output_count();
        let used = used_input_count[node_idx];

        // (a) Historical exact‐match check
        if used != in_count {
            return Err(NetworkError::InvalidConfiguration {
                details: format!(
                    "Node #{} operator expects {} inputs, but wired edges used {}",
                    node_idx, in_count, used
                ),
            });
        }

        // (b, c) For each input port in [0..in_count], check no double‐feeding & required connections
        for i in 0..in_count {
            if input_usage[node_idx][i] > 1 {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!(
                        "Node #{} input port {} is fed by multiple edges ({}), which is not allowed",
                        node_idx, i, input_usage[node_idx][i]
                    ),
                });
            }
            if op.input_port_connection_required(i) && input_usage[node_idx][i] == 0 {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!(
                        "Node #{} input port {} is required but has no incoming edge",
                        node_idx, i
                    ),
                });
            }
        }

        // (d) For each output port in [0..out_count], if it’s required => must have at least one consumer
        //     Note that fan‐out is allowed, so we only check output_usage[node_idx][port] > 0
        for o in 0..out_count {
            if op.output_port_connection_required(o) && output_usage[node_idx][o] == 0 {
                return Err(NetworkError::InvalidConfiguration {
                    details: format!(
                        "Node #{} output port {} is required but has no downstream edge",
                        node_idx, o
                    ),
                });
            }
        }
    }

    Ok(())
}

// =====================
// wire_up_network_tests.rs
// =====================
#[cfg(test)]
mod wire_up_network_tests {

    use super::*; // bring in wire_up_network, Network, operator types, etc.

    /// 1) Single node => operator=ConstantOp => input_count=0 => output_count=1 => no edges => OK.
    ///    Basic check that we allocate 1 output slot and do not require any input edges.
    #[test]
    fn test_wire_up_single_constantop_ok() -> Result<(), NetworkError> {

        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(100));

        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0])
            .edges(vec![])
            .build()
            .unwrap();

        // wire_up => ensures we have an Arc for output port=0
        wire_up_network(&mut net)?;

        assert!(net.nodes()[0].outputs()[0].is_some());
        assert!(net.nodes()[0].outputs()[1].is_none());

        // input_count=0 => so no edges => inputs=all None
        for i in 0..4 {
            assert!(net.nodes()[0].inputs()[i].is_none());
        }

        Ok(())
    }

    /// 2) Single node => operator=AddOp => input_count=1 => output_count=1 => no edges => Mismatch => error.
    #[test]
    fn test_wire_up_single_addop_mismatch() {
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => AddOp::new(10));
        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0])
            .edges(vec![])
            .build()
            .unwrap();

        let res = wire_up_network(&mut net);
        assert!(res.is_err());
        if let Err(NetworkError::InvalidConfiguration{ details }) = res {
            assert!(
                details.contains("expects 1 inputs, but wired edges used 0"),
                "Expected an input_count mismatch error, got: {}",
                details
            );
        }
    }

    /// 3) Two nodes => Node0=ConstantOp => Node1=AddOp => single edge => OK.
    ///    We specifically test that Node1 sees input_count=1 used => no mismatch.
    #[test]
    fn test_wire_up_two_nodes_ok() -> Result<(), NetworkError> {
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(42));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(5));
        let e = edge!(0:0 -> 1:0);

        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1])
            .edges(vec![e])
            .build()
            .unwrap();

        wire_up_network(&mut net)?;

        // Node0 => out_count=1 => outputs[0].is_some
        assert!(net.nodes()[0].outputs()[0].is_some());
        // Node1 => in_count=1 => inputs[0].is_some
        assert!(net.nodes()[1].inputs()[0].is_some());

        Ok(())
    }

    /// 4) Multiple edges from the same output => Node0 => Node1, Node2. 
    ///    E.g. Node0=ConstantOp => Node1=AddOp => Node2=MultiplyOp => 
    ///    We do not fail if we fan‐out from node0’s output to multiple nodes. 
    ///    Each node sees input_count=1 used => no mismatch.
    #[test]
    fn test_wire_up_fanout_ok() -> Result<(), NetworkError> {
        // Node0 => out_count=1 => edges => Node1( input_count=1 ) and Node2( input_count=1 )
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(10));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(100));
        let n2: NetworkNode<TestWireIO<i32>> = node!(2 => MultiplyOp::new(2));

        let e1 = edge!(0:0 -> 1:0);
        let e2 = edge!(0:0 -> 2:0);

        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1, n2])
            .edges(vec![e1, e2])
            .build()
            .unwrap();

        wire_up_network(&mut net)?;
        // Node0 => outputs[0].some => used by both Node1, Node2
        assert!(net.nodes()[0].outputs()[0].is_some());
        // Node1 => inputs[0].some
        assert!(net.nodes()[1].inputs()[0].is_some());
        // Node2 => inputs[0].some
        assert!(net.nodes()[2].inputs()[0].is_some());
        Ok(())
    }

    /// 5) The same input port is fed by multiple edges => that’s not allowed => we expect an error
    ///    because used_input_count would be incremented more than once.
    #[test]
    fn test_wire_up_double_feed_same_input_port() {
        // Node1 => input_count=1 => but we attempt 2 edges => Node0 -> Node1:0, Node2 -> Node1:0
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(42));
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => AddOp::new(5));
        let n2: NetworkNode<TestWireIO<i32>> = node!(2 => ConstantOp::new(123));

        // Edges => Node0:0->Node1:0, and Node2:0->Node1:0
        // This means node1's input=port0 is fed from two arcs => 
        // wire_up_network => used_input_count[1]=2 while operator requires 1 => error.
        let e0 = edge!(0:0 -> 1:0);
        let e1 = edge!(2:0 -> 1:0);

        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0, n1, n2])
            .edges(vec![e0, e1])
            .build()
            .unwrap();

        let res = wire_up_network(&mut net);
        assert!(res.is_err());
        if let Err(NetworkError::InvalidConfiguration{details}) = res {
            assert!(
                details.contains("expects 1 inputs, but wired edges used 2"),
                "Expected mismatch for node=1 with 2 edges into the same port. Got: {}",
                details
            );
        }
    }

    /// 6) The operator is declared with .output_count=2, but we connect no edges to the 2nd output => that’s allowed
    ///    as long as the operator’s input_count is matched. 
    ///    We can create a custom operator that returns .input_count=1, .output_count=2 to test it.
    #[test]
    fn test_wire_up_operator_with_2_outputs_but_only_1_used() -> Result<(), NetworkError> {
        // Node0 => DoubleOutOp => 2 outputs => we connect only the 0th to Node1 => that’s fine 
        // as long as wire_up sees no mismatch in input_count. 
        let n0: NetworkNode<TestWireIO<i32>> = node!(0 => DoubleOutOp::default());
        let n1: NetworkNode<TestWireIO<i32>> = node!(1 => MultiplyOp::new(2));
        // Edges => 0:0 -> 1:0. We leave out 0:1 => not used. 
        // Node0’s input_count=1 => we must feed it from e.g. a ConstantOp or remove the input altogether. 
        // But that requires node0’s input_count=1 => Mismatch again. 
        // So let’s chain a new Node(-1) => a short hack:
        let nX: NetworkNode<TestWireIO<i32>> = node!(999 => ConstantOp::new(77));
        // Edge => 999:0 -> 0:0 => so node0 gets input
        let eX = edge!(999:0 -> 0:0);

        let e0 = edge!(0:0 -> 1:0);

        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![nX, n0, n1]) // nodeX=0th in array, node0=1, node1=2 => watch out for indexing mismatch
            // Actually we must be consistent with indexes => let’s do the real approach:
            .edges(vec![ eX, e0 ])
            .build()
            .unwrap();

        // Wait, we are referencing node=999 in the builder but we only pushed 3 nodes => that’s an out_of_range.
        // Instead, let’s reorder:
        // node0 => constant
        // node1 => double-out
        // node2 => multiply
        // edges => (0->1:0) => feed double-out input, (1:0->2:0) => only first output used
        let n0const: NetworkNode<TestWireIO<i32>> = node!(0 => ConstantOp::new(77));
        let n1dbl:   NetworkNode<TestWireIO<i32>> = node!(1 => DoubleOutOp::default());
        let n2mul:   NetworkNode<TestWireIO<i32>> = node!(2 => MultiplyOp::new(2));
        let eA = edge!(0:0 -> 1:0); // feed the double-out
        let eB = edge!(1:0 -> 2:0); // use only the first output
        let mut net = NetworkBuilder::<TestWireIO<i32>>::default()
            .nodes(vec![n0const, n1dbl, n2mul])
            .edges(vec![eA, eB])
            .build()
            .unwrap();

        wire_up_network(&mut net)?;
        // Node1 => out_count=2 => so we have outputs[0].some, outputs[1].some 
        assert!(net.nodes()[1].outputs()[0].is_some());
        assert!(net.nodes()[1].outputs()[1].is_some());
        // Node2 => in_count=1 => inputs[0].some => from node1:0
        assert!(net.nodes()[2].inputs()[0].is_some());
        // That second output node1:1 is allocated but not used => that’s allowed => no error
        Ok(())
    }
}
