// ---------------- [ File: tests/test_integration_crazy_wiring.rs ]
//! tests/test_integration_crazy_wiring.rs

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
fn test_integration_crazy_wiring() -> Result<(), NetworkError> {

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

        // 2) Define 8 nodes
        // Node0 => Constant(5)
        // Node1 => Add(+10)
        // Node2 => DoubleOutOp => 2 outputs
        // Node3 => Multiply(4)
        // Node4 => Add(+5)
        // Node5 => Merge2 => sums input0 + input1 => 1 output
        // Node6 => Multiply(-1) => final
        // Node7 => SingleValOp (just for fun, not feeding final)
        vec![
            node!(0 => ConstantOp::new(5)),
            node!(1 => AddOp::new(10)),
            node!(2 => DoubleOutOp::default()),
            node!(3 => MultiplyOp::new(4)),
            node!(4 => AddOp::new(5)),
            node!(5 => Merge2Op::default()),
            node!(6 => MultiplyOp::new(-1)),
            node!(7 => SingleValOp::default()),
        ],

        // 3) Crazy edges:
        // Node0:0 -> Node1:0 => (5 => 15)
        // Node1:0 -> Node2:0 => (15 => out0=15, out1=115)
        //
        // Then from Node2 => out0=port0 => Node3:0 => (15 => multiply(4)=60)
        // Then from Node2 => out1=port1 => Node4:0 => (115 => +5=120)
        //
        // Next we merge node3 => node4 into node5 => Merge2 => out= 60 + 120=180
        // Then node5 => node6 => multiply(-1) => final=-180
        // Meanwhile, node2: out1 => node7 => SingleValOp => always 777 => doesn't feed the final
        //
        // So the final is node6 => output => we expect -180
        vec![
            edge!(0:0 -> 1:0), // constant(5) => add(10)=15
            edge!(1:0 -> 2:0), // => doubleOut => out0=15, out1=115
            edge!(2:0 -> 3:0), // => multiply(4)=60
            edge!(2:1 -> 4:0), // => add(5)=120
            edge!(3:0 -> 5:0), // => feed merge2: input0=60
            edge!(4:0 -> 5:1), // => feed merge2: input1=120
            edge!(5:0 -> 6:0), // => multiply(-1)
        ]
    );

    // 5) Put in Arc<Mutex>
    let net_arc = Arc::new(AsyncMutex::new(network));

    // 6) Execute
    let (perf, _maybe_stream) = scheduler.execute_network(net_arc.clone())?;
    info!("test_integration_crazy_wiring => perf={perf:?}");

    // 7) Read final node’s output => node6 => multiply(-1)
    // We expect node6 output => -180
    let final_val = block_on(async {
        let guard = net_arc.lock().await;
        let node6 = &guard.nodes()[6];
        let out_arc = node6.outputs()[0].clone().expect("node6 has an output arc");
        out_arc.read().await.clone()
    });
    assert_eq!(test_wire_port0_into!{final_val => i32}, -180, "Expected final to be -180");

    // Also, check node7 => SingleValOp => always 777
    let node7_val = block_on(async {
        let guard = net_arc.lock().await;
        let node7 = &guard.nodes()[7];
        let arc = node7.outputs()[0].clone().expect("node7 has output arc");
        arc.read().await.clone()
    });
    assert_eq!(test_wire_port0_into!{node7_val => i32}, 777, "SingleValOp always sets 777 on out0");

    Ok(())
}
