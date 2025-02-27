// ---------------- [ File: src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{wire_up}
x!{test_wire}
x!{edge}
x!{node}
x!{network}
x!{validate}

#[cfg(test)]
mod large_network_integration_tests {
    use super::*;
    use hydro2_network_wire_derive::*;
    use futures::executor::block_on; // or some async runtime approach

    #[traced_test]
    fn test_large_network_flow() -> Result<(), NetworkError> {

        let net: Network<TestWireIO<i32>> = network!{
            // We create 8 nodes in a chain/fan-out style:
            //   n0(Constant(10)) => n1(AddOp(5)) => n2(MultiplyOp(3)) => n3(NoOp)
            //   also from n2 => n4(SplitAndDoubleOp) => out => n5(AddOp(100)) => ...
            //   etc. We can get creative.
            vec![
                // no input, output=10
                node!(0 => ConstantOp::new(10)),   

                // input=10 => output=15
                node!(1 => AddOp::new(5)),         

                // input=15 => output=45
                node!(2 => MultiplyOp::new(3)),    

                node!(3 => SingleChannelPassthroughOperator::<i32>::with_name("NoOp3")),

                node!(4 => SplitAndDoubleOp::default()),

                // feed from n4:0 or n4:1
                node!(5 => AddOp::new(100)),       

                // optional => invert something
                node!(6 => MultiplyOp::new(-1)),   

                node!(7 => SingleValOp::default()),
            ],
            // Edges:
            // n0:0 -> n1:0 -> n2:0 -> n3:0 -> n4:0 => fan out => n5, n6, n7 or something
            vec![
                edge!(0:0 -> 1:0),
                edge!(1:0 -> 2:0),
                edge!(2:0 -> 3:0),
                edge!(3:0 -> 4:0), // input_count=1 => no problem

                // n4 => output_count=2 => we can feed n5, n6 from the two outputs
                edge!(4:0 -> 5:0), // out0 => input0 => n5 => yields something
                edge!(4:1 -> 6:0), // out1 => input0 => n6 => yields something

                // n7 stands alone
            ]
        };

        eprintln!("net: {:#?}", net);

        // Now we want to actually “execute” the final pipeline. 
        // Because we have no scheduling system here, let's do a manual BFS style:
        //  - find a node with no inputs => run => find any node that is now complete => run => etc.
        // Or we can just do a topological order and call node[i].execute().await in that order.

        // Topological order: [0,1,2,3,4,5,6,7]
        // We'll do an async block_on approach:
        block_on(async {
            for i in 0..8 {
                net.nodes()[i].execute().await?;
            }
            Ok::<(), NetworkError>(())
        })?;

        eprintln!("net, post exec: {:#?}", net);

        // Now let's see what happened:
        // Node0 => output=10
        // Node1 => input=10 => output=15
        // Node2 => input=15 => output=45
        // Node3 => input=45 => output=None => (NoOp)
        // Node4 => input=45 => out0=45, out1=90
        // Node5 => input=45 => output=145
        // Node6 => input=90 => output=-90
        // Node7 => input=145 => output=777 (SingleValOp doesn’t use input)
        
        // Let's verify that manually:
        // We can read back each node’s output arcs if we want:
        let node7_output_arc_opt = net.nodes()[7].outputs()[0].clone();
        assert!(node7_output_arc_opt.is_some());
        let node7_arc = node7_output_arc_opt.unwrap();
        let node7_val = block_on(async { node7_arc.read().await.clone() });
        // singleValOp => forced=777
        assert_eq!(node7_val, TestWireIO::SingleValOpIO(SingleValOpIO::Output0(777)));

        // Also we can confirm node6 => -90
        let node6_arc = net.nodes()[6].outputs()[0].clone().unwrap();
        let node6_val = block_on(async { node6_arc.read().await.clone() });
        assert_eq!(node6_val, TestWireIO::MultiplyOpIO(MultiplyOpIO::Output0(-90)));

        // Node5 => out=145
        let node5_arc = net.nodes()[5].outputs()[0].clone().unwrap();
        let node5_val = block_on(async { node5_arc.read().await.clone() });
        assert_eq!(node5_val, TestWireIO::AddOpIO(AddOpIO::Output0(145)));

        // Node4 => out0=45, out1=90
        let node4_arc0 = net.nodes()[4].outputs()[0].clone().unwrap();
        let node4_val0 = block_on(async { node4_arc0.read().await.clone() });
        assert_eq!(node4_val0, TestWireIO::SplitAndDoubleOpIO(SplitAndDoubleOpIO::Output0(45)));
        let node4_arc1 = net.nodes()[4].outputs()[1].clone().unwrap();
        let node4_val1 = block_on(async { node4_arc1.read().await.clone() });
        assert_eq!(node4_val1, TestWireIO::SplitAndDoubleOpIO(SplitAndDoubleOpIO::Output1(90)));

        Ok(())
    }
}
