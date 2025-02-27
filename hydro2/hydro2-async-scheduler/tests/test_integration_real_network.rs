// ---------------- [ File: tests/test_integration_real_network.rs ]
//! tests/test_integration_real_network.rs

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
fn test_integration_real_network() -> Result<(), NetworkError> {

    // We’ll build + run in a synchronous test, 
    // but we can also do `#[tokio::test]` if desired.

    // 1) Build the scheduler with immediate scheduling + concurrency=3
    let cfg = AsyncSchedulerConfigBuilder::default()
        .batching_strategy(BatchingStrategy::Immediate)
        .max_parallelism(3_usize)
        .enable_streaming(false)
        .build()
        .unwrap();
    let scheduler = AsyncScheduler::with_config(cfg);

    // 4) Build the network
    // 5) Put it in an Arc<Mutex> so the scheduler can use it
    let net_arc: Arc<AsyncMutex<Network<TestWireIO<i32>>>> = Arc::new(AsyncMutex::new(network!{

        // 2) Build our 7 nodes => each has 1 output buffer if it’s an integer operator, 
        //    or 1 output buffer if it’s a string operator in the last node, etc.
        vec![

            // Node0 => Constant(42) => output0 => i32
            node!(0 => ConstantOp::new(42)),

            // Node1 => Add(+100) => input0 => i32, output0 => i32
            node!(1 => AddOp::new(100)),

            // Node2 => Multiply(x3)
            node!(2 => MultiplyOp::new(3)),

            // Node3 => Add(-20) => effectively subtract 20
            node!(3 => AddOp::new(-20)),

            // Node4 => Multiply(x2)
            node!(4 => MultiplyOp::new(2)),

            // Node5 => NoOp, i32->i32
            node!(5 => SingleChannelPassthroughOperator::<i32>::with_name("NoOp5")),
        ],
        // 3) Build edges => wiring up output0 of node0 -> input0 of node1, etc.
        //    We must match indices: node(0).output(0)->node(1).input(0) => edge!(0:0 -> 1:0)
        vec![

            // Node0 => Node1
            edge!(0:0 -> 1:0),
            // Node1 => Node2
            edge!(1:0 -> 2:0),
            // Node2 => Node3
            edge!(2:0 -> 3:0),
            // Node3 => Node4
            edge!(3:0 -> 4:0),
            // Node4 => Node5
            edge!(4:0 -> 5:0),
        ]
    }));

    // 6) Execute
    let (perf, _maybe_stream) = scheduler.execute_network(net_arc.clone())?;
    info!("test_integration_real_network => perf={:?}", perf);

    // 7) After scheduling => check final node’s output
    //    Node6 => has 1 output => a String => we want "Value=812"
    let final_val = block_on(async {
        let mut guard = net_arc.lock().await;
        let node5 = &guard.nodes_mut()[5];
        // node6.output_buffers()[0] is Arc<String>
        let s = &node5.outputs()[0];
        s.clone().unwrap().read().await.clone() // Arc<String>
    });

    assert_eq!(test_wire_port0_into!{final_val => i32}, 812, "Expected final_val => 812");
    
    Ok(())
}
