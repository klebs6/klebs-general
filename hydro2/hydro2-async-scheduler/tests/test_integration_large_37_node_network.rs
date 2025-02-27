// ---------------- [ File: hydro2-async-scheduler/tests/test_integration_large_37_node_network.rs ]
//! tests/test_integration_large_37_node_network.rs

#![allow(clippy::needless_return)]
#![allow(clippy::redundant_closure)]

use hydro2_mock::*;
use hydro2_3p::*;
use hydro2_network::*;
use hydro2_network_performance::*;
use hydro2_operator::*;
use hydro2_basic_operators::*;
use hydro2_async_scheduler::*;

#[test]
fn test_integration_37_nodes_crazy_wiring() -> Result<(), NetworkError> {

    use std::sync::Arc;
    use futures::executor::block_on;

    // 1) Build scheduler config => concurrency=4
    let cfg = AsyncSchedulerConfigBuilder::default()
        .batching_strategy(BatchingStrategy::Immediate)
        .max_parallelism(4_usize)
        .enable_streaming(false)
        .build()
        .unwrap();
    let scheduler = AsyncScheduler::with_config(cfg);

    // 4) Build the network
    let network: Network<TestWireIO<i32>> = network!(
        // 2) Define 37 nodes (0..36). We'll do a mostly “chain + fan‐in/out” approach.
        //    Some are not connected at all, just to ensure they don't break anything.
        //    We'll note the main path that eventually feeds node36, plus some side nodes.
        vec![

            // Node0 => Constant(1)
            node!(0 => ConstantOp::new(1)),
            // Node1 => Add(+2)
            node!(1 => AddOp::new(2)),
            // Node2 => Multiply(x3)
            node!(2 => MultiplyOp::new(3)),
            // Node3 => Add(+4)
            node!(3 => AddOp::new(4)),
            // Node4 => Multiply(x5)
            node!(4 => MultiplyOp::new(5)),
            // Node5 => Add(+6)
            node!(5 => AddOp::new(6)),
            // Node6 => DoubleOutOp => out0=val, out1=val+100
            node!(6 => DoubleOutOp::default()),
            // Node7 => Merge2 => sum(in0 + in1)
            node!(7 => Merge2Op::default()),
            // Node8 => Add(+10)
            node!(8 => AddOp::new(10)),
            // Node9 => Multiply(x2)
            node!(9 => MultiplyOp::new(2)),
            // Node10 => SingleVal => always 777 (not used by chain)
            node!(10 => SingleValOp::default()),
            // Node11 => Add(+3)
            node!(11 => AddOp::new(3)),
            // Node12 => Multiply(x2)
            node!(12 => MultiplyOp::new(2)),
            // Node13 => Add(+5)
            node!(13 => AddOp::new(5)),
            // Node14 => DoubleOutOp
            node!(14 => DoubleOutOp::default()),
            // Node15 => Merge2
            node!(15 => Merge2Op::default()),
            // Node16 => Add(+10)
            node!(16 => AddOp::new(10)),
            // Node17 => Multiply(x3)
            node!(17 => MultiplyOp::new(3)),
            // Node18 => IncrementOperator => adds +1 to input
            node!(18 => IncrementOperator::default()),
            // Node19 => DoubleOutOp
            node!(19 => DoubleOutOp::default()),
            // Node20 => Merge2
            node!(20 => Merge2Op::default()),
            // Node21 => Add(+100)
            node!(21 => AddOp::new(100)),
            // Node22 => Multiply(x2)
            node!(22 => MultiplyOp::new(2)),
            // Node23 => SingleVal => 777 (unused)
            node!(23 => SingleValOp::default()),
            // Node24 => DoubleOutOp
            node!(24 => DoubleOutOp::default()),
            // Node25 => Merge2
            node!(25 => Merge2Op::default()),
            // Node26 => Add(+1000)
            node!(26 => AddOp::new(1000)),
            // Node27 => Multiply(x2)
            node!(27 => MultiplyOp::new(2)),
            // Node28 => SingleVal => 777 (unused)
            node!(28 => SingleValOp::default()),
            // Node29 => SingleChannelPassthrough => pass input unchanged
            node!(29 => SingleChannelPassthroughOperator::<i32>::with_name("Passthrough29")),
            // Node30 => DoubleOutOp
            node!(30 => DoubleOutOp::default()),
            // Node31 => Merge2
            node!(31 => Merge2Op::default()),
            // Node32 => Add(+2)
            node!(32 => AddOp::new(2)),
            // Node33 => Multiply(-1)
            node!(33 => MultiplyOp::new(-1)),
            // Node34 => SingleVal => 777 (unused)
            node!(34 => SingleValOp::default()),
            // Node35 => Add(+999)
            node!(35 => AddOp::new(999)),
            // Node36 => Multiply(x100) => final node
            node!(36 => MultiplyOp::new(100)),
        ],
        // 3) The edges. The main chain is:
        //
        // 0->1->2->3->4->5->6 => DoubleOut => out0->7:0, out1->7:1 => merges => 7->8->9->11->12->13->14 => out0->15:0, out1->15:1 => merges => 15->16->17->18->19 => out0->20:0, out1->20:1 => merges => 20->21->22->24 => out0->25:0, out1->25:1 => merges => 25->26->27->29->30 => out0->31:0, out1->31:1 => merges => 31->32->33->35->36 => final
        //
        // The nodes 10, 23, 28, 34 are SingleValOp(=777) but not connected anywhere, just present.
        vec![
            edge!(0:0 -> 1:0),  // 1 => Add(2)
            edge!(1:0 -> 2:0),  // 2 => Multiply(3)
            edge!(2:0 -> 3:0),  // 3 => Add(4)
            edge!(3:0 -> 4:0),  // 4 => Multiply(5)
            edge!(4:0 -> 5:0),  // 5 => Add(6)
            edge!(5:0 -> 6:0),  // 6 => DoubleOut => out0=..., out1=...+100
                                // DoubleOut node6 => merges at node7
            edge!(6:0 -> 7:0),
            edge!(6:1 -> 7:1),
            edge!(7:0 -> 8:0),  // 8 => Add(10)
            edge!(8:0 -> 9:0),  // 9 => Multiply(2)
            edge!(9:0 -> 11:0), // 11 => Add(3)
            edge!(11:0 -> 12:0),
            edge!(12:0 -> 13:0),
            edge!(13:0 -> 14:0), // 14 => DoubleOut
                                 // DoubleOut node14 => merges at node15
            edge!(14:0 -> 15:0),
            edge!(14:1 -> 15:1),
            edge!(15:0 -> 16:0),
            edge!(16:0 -> 17:0),
            edge!(17:0 -> 18:0), // 18 => increment => +1
            edge!(17:0 -> 18:1), // 18 => increment => +1
            edge!(17:0 -> 18:2), // 18 => increment => +1
            edge!(17:0 -> 18:3), // 18 => increment => +1
            edge!(18:0 -> 19:0), // 19 => DoubleOut
            edge!(19:0 -> 20:0),
            edge!(19:1 -> 20:1),
            edge!(20:0 -> 21:0),
            edge!(21:0 -> 22:0),
            edge!(22:0 -> 24:0), // 24 => DoubleOut
                                 // DoubleOut node24 => merges at node25
            edge!(24:0 -> 25:0),
            edge!(24:1 -> 25:1),
            edge!(25:0 -> 26:0),
            edge!(26:0 -> 27:0),
            edge!(27:0 -> 29:0), // 29 => SingleChannelPassthrough
            edge!(29:0 -> 30:0), // 30 => DoubleOut
            edge!(30:0 -> 31:0),
            edge!(30:1 -> 31:1),
            edge!(31:0 -> 32:0),
            edge!(32:0 -> 33:0),
            edge!(33:0 -> 35:0),
            edge!(35:0 -> 36:0), // final Note: node10,23,28,34 are not connected => no edges
        ]
    );

    // 5) Put in Arc<Mutex> so scheduler can run it
    let net_arc = Arc::new(AsyncMutex::new(network));

    // 6) Execute
    let (perf, _maybe_stream) = scheduler.execute_network(net_arc.clone())?;
    info!("test_integration_37_nodes_crazy_wiring => perf={perf:?}");

    // 7) The final node is node36 => Multiply(100). Let’s read that output:
    // We expect -21,294,300 if we carefully computed each step (see the chain logic).
    let final_val = block_on(async {
        let guard = net_arc.lock().await;
        let node36 = &guard.nodes()[36];
        let arc_opt = node36.outputs()[0].clone().expect("node36 output");
        arc_opt.read().await.clone()
    });

    assert_eq!(test_wire_port0_into!{final_val => i32}, -21294300, "Expected final of -21,294,300");
    Ok(())
}
