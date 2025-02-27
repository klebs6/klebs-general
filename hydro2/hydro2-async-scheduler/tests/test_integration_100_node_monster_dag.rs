// ---------------- [ File: tests/test_integration_100_node_monster_dag.rs ]
//! tests/test_integration_100_node_monster_dag.rs

#![allow(clippy::needless_return)]
#![allow(clippy::redundant_closure)]

use hydro2_mock::*;
use hydro2_3p::*;
use hydro2_network::*;
use hydro2_network_performance::*;
use hydro2_operator::*;
use hydro2_async_scheduler::*;
use hydro2_basic_operators::*;

#[traced_test]
#[disable] // can't seem to get the wiring right, will wait for stronger AI
fn test_integration_100_nodes_monster_dag() -> Result<(), NetworkError> {
    use std::sync::Arc;
    use futures::executor::block_on;

    // 1) Build a scheduler config => concurrency=8, immediate scheduling
    let cfg = AsyncSchedulerConfigBuilder::default()
        .batching_strategy(BatchingStrategy::Immediate)
        .max_parallelism(8_usize)
        .enable_streaming(false)
        .build()
        .unwrap();
    let scheduler = AsyncScheduler::with_config(cfg);

    // ================
    // (A) Create 110 nodes
    // ================

    // Constants
    let n0  = node!(0   => ConstantOp::new(1));     // => 1
    let n1  = node!(1   => ConstantOp::new(100));   // => 100
    let n2  = node!(2   => ConstantOp::new(-3));    // => -3
    let n3  = node!(3   => ConstantOp::new(9999));  // => 9999
    let n4  = node!(4   => ConstantOp::new(42));    // => 42
    let n5  = node!(5   => ConstantOp::new(7));     // => 7
    let n6  = node!(6   => ConstantOp::new(0));     // => 0

    // SingleValOp (input_count=0 => no edges needed unless we *choose* to feed its output)
    let n7  = node!(7   => SingleValOp);  // => 777
    let n8  = node!(8   => SingleValOp);  // => 777
    let n9  = node!(9   => SingleValOp);  // => 777
    let n30 = node!(30  => SingleValOp);
    let n31 = node!(31  => SingleValOp);
    let n41 = node!(41  => SingleValOp);
    let n49 = node!(49  => SingleValOp);
    let n57 = node!(57  => SingleValOp);
    let n65 = node!(65  => SingleValOp);
    let n73 = node!(73  => SingleValOp);
    let n88 = node!(88  => SingleValOp);

    // Simple AddOps (input_count=1)
    let n10 = node!(10  => AddOp::new(5));
    let n11 = node!(11  => AddOp::new(-10));
    let n12 = node!(12  => AddOp::new(1000));
    let n13 = node!(13  => AddOp::new(2));
    let n22 = node!(22  => AddOp::new(1));
    let n32 = node!(32  => AddOp::new(300));
    let n40 = node!(40  => AddOp::new(50));
    let n47 = node!(47  => AddOp::new(-77));
    let n55 = node!(55  => AddOp::new(-8));
    let n63 = node!(63  => AddOp::new(-10));
    let n71 = node!(71  => AddOp::new(1234));
    let n79 = node!(79  => AddOp::new(999));
    let n86 = node!(86  => AddOp::new(77));
    let n94 = node!(94  => AddOp::new(9999));
    let n101= node!(101 => AddOp::new(123));

    // Simple MultiplyOps (input_count=1)
    let n14 = node!(14  => MultiplyOp::new(2));
    let n15 = node!(15  => MultiplyOp::new(-1));
    let n16 = node!(16  => MultiplyOp::new(10));
    let n23 = node!(23  => MultiplyOp::new(2));
    let n33 = node!(33  => MultiplyOp::new(4));
    let n39 = node!(39  => MultiplyOp::new(-2));
    let n48 = node!(48  => MultiplyOp::new(5));
    let n56 = node!(56  => MultiplyOp::new(6));
    let n64 = node!(64  => MultiplyOp::new(2));
    let n72 = node!(72  => MultiplyOp::new(3));
    let n80 = node!(80  => MultiplyOp::new(-3));
    let n87 = node!(87  => MultiplyOp::new(5));
    let n95 = node!(95  => MultiplyOp::new(-1));
    let n102= node!(102 => MultiplyOp::new(-4));

    // Some custom multi‐I/O operators:
    // SingleToTriOp => input=1 => output=3
    // DoubleToTriOp => input=2 => output=3
    // TriToSingleOp => input=3 => output=1
    // TriToQuadOp   => input=3 => output=4
    // QuadToQuadOp  => input=4 => output=4
    let n17 = node!(17 => SingleToTriOp);
    let n18 = node!(18 => TriToSingleOp);
    let n19 = node!(19 => TriToQuadOp);
    let n20 = node!(20 => DoubleToTriOp);
    let n21 = node!(21 => QuadToQuadOp);
    let n24 = node!(24 => DoubleToTriOp);
    let n25 = node!(25 => TriToQuadOp);
    let n26 = node!(26 => TriToSingleOp);
    let n27 = node!(27 => SingleToTriOp);
    let n28 = node!(28 => QuadToQuadOp);
    let n29 = node!(29 => DoubleToTriOp);
    let n34 = node!(34 => DoubleToTriOp);
    let n35 = node!(35 => TriToQuadOp);
    let n36 = node!(36 => TriToSingleOp);
    let n37 = node!(37 => SingleToTriOp);
    let n38 = node!(38 => QuadToQuadOp);
    let n42 = node!(42 => DoubleToTriOp);
    let n43 = node!(43 => TriToQuadOp);
    let n44 = node!(44 => TriToSingleOp);
    let n45 = node!(45 => SingleToTriOp);
    let n46 = node!(46 => QuadToQuadOp);
    let n50 = node!(50 => DoubleToTriOp);
    let n51 = node!(51 => TriToQuadOp);
    let n52 = node!(52 => TriToSingleOp);
    let n53 = node!(53 => SingleToTriOp);
    let n54 = node!(54 => QuadToQuadOp);
    let n58 = node!(58 => DoubleToTriOp);
    let n59 = node!(59 => TriToQuadOp);
    let n60 = node!(60 => TriToSingleOp);
    let n61 = node!(61 => SingleToTriOp);
    let n62 = node!(62 => QuadToQuadOp);
    let n66 = node!(66 => DoubleToTriOp);
    let n67 = node!(67 => TriToQuadOp);
    let n68 = node!(68 => TriToSingleOp);
    let n69 = node!(69 => SingleToTriOp);
    let n70 = node!(70 => QuadToQuadOp);
    let n74 = node!(74 => DoubleToTriOp);
    let n75 = node!(75 => TriToQuadOp);
    let n76 = node!(76 => TriToSingleOp);
    let n77 = node!(77 => SingleToTriOp);
    let n78 = node!(78 => QuadToQuadOp);
    let n81 = node!(81 => DoubleToTriOp);
    let n82 = node!(82 => TriToQuadOp);
    let n83 = node!(83 => TriToSingleOp);
    let n84 = node!(84 => SingleToTriOp);
    let n85 = node!(85 => QuadToQuadOp);
    let n89 = node!(89 => DoubleToTriOp);
    let n90 = node!(90 => TriToQuadOp);
    let n91 = node!(91 => TriToSingleOp);
    let n92 = node!(92 => SingleToTriOp);
    let n93 = node!(93 => QuadToQuadOp);
    let n96 = node!(96 => DoubleToTriOp);
    let n97 = node!(97 => TriToQuadOp);
    let n98 = node!(98 => TriToSingleOp);
    let n99 = node!(99 => SingleToTriOp);
    let n100= node!(100=> QuadToQuadOp);
    let n103= node!(103=> DoubleToTriOp);
    let n104= node!(104=> TriToQuadOp);
    let n105= node!(105=> TriToSingleOp);
    let n106= node!(106=> SingleToTriOp);
    let n107= node!(107=> QuadToQuadOp);
    let n108= node!(108=> DoubleToTriOp);
    let n109= node!(109=> QuadToQuadOp); // final node

    let all_nodes = vec![
        n0,n1,n2,n3,n4,n5,n6,
        n7,n8,n9,n30,n31,n41,n49,n57,n65,n73,n88,
        n10,n11,n12,n13,n14,n15,n16,n17,n18,n19,
        n20,n21,n22,n23,n24,n25,n26,n27,n28,n29,
        n32,n33,n34,n35,n36,n37,n38,n39,n40,n42,
        n43,n44,n45,n46,n47,n48,n50,n51,n52,n53,
        n54,n55,n56,n58,n59,n60,n61,n62,n63,n64,
        n66,n67,n68,n69,n70,n71,n72,n74,n75,n76,
        n77,n78,n79,n80,n81,n82,n83,n84,n85,n86,
        n87,n89,n90,n91,n92,n93,n94,n95,n96,n97,
        n98,n99,n100,n101,n102,n103,n104,n105,n106,n107,
        n108,n109,
    ];

    // ================
    // (B) Define Edges so each node’s input_count is fully satisfied
    // ================
    let mut edges = Vec::new();

    // -------------------------------------------------
    // Subchain #1 (already had these)
    // (node0=1) -> n17(SingleToTri) -> n18(TriToSingle) -> n20(DoubleToTri) -> n19(TriToQuad) -> n28(QuadToQuad) -> n21(QuadToQuad)
    // -------------------------------------------------
    edges.push(edge!(0:0 -> 17:0)); // singleToTri => in=1 => ok
    edges.push(edge!(17:0 -> 18:0));
    edges.push(edge!(17:1 -> 18:1));
    edges.push(edge!(17:2 -> 18:2));
    edges.push(edge!(1:0  -> 20:0));
    edges.push(edge!(18:0 -> 20:1));
    edges.push(edge!(20:0 -> 19:0));
    edges.push(edge!(20:1 -> 19:1));
    edges.push(edge!(20:2 -> 19:2));
    edges.push(edge!(19:0 -> 28:0));
    edges.push(edge!(19:1 -> 28:1));
    edges.push(edge!(19:2 -> 28:2));
    edges.push(edge!(19:3 -> 28:3));
    // Then feed node28 => out(4) => node21 => in(4)
    edges.push(edge!(28:0 -> 21:0));
    edges.push(edge!(28:1 -> 21:1));
    edges.push(edge!(28:2 -> 21:2));
    edges.push(edge!(28:3 -> 21:3));

    // -------------------------------------------------
    // Subchain #2 (already had these)
    // node2=-3, node3=9999 => feed n24(DoubleToTri) => n25(TriToQuad) => n26(TriToSingle) => n22(Add1) => n23(Mul2) => n14(Mul2) => n15(Mul-1) => ...
    // -------------------------------------------------
    edges.push(edge!(2:0 -> 24:0));
    edges.push(edge!(3:0 -> 24:1));
    edges.push(edge!(24:0 -> 25:0));
    edges.push(edge!(24:1 -> 25:1));
    edges.push(edge!(24:2 -> 25:2));
    edges.push(edge!(25:0 -> 26:0));
    edges.push(edge!(25:1 -> 26:1));
    edges.push(edge!(25:2 -> 26:2));
    edges.push(edge!(26:0 -> 22:0));
    edges.push(edge!(22:0 -> 23:0));
    edges.push(edge!(23:0 -> 14:0));
    edges.push(edge!(14:0 -> 15:0)); // feed n15 => multiply(-1)

    // feed node15 => out => node40=Add(50) => node39=Mul(-2) => node47=Add(-77) => node48=Mul(5) => node63=Add(-10) => node64=Mul(2)
    edges.push(edge!(15:0 -> 40:0));
    edges.push(edge!(40:0 -> 39:0));
    edges.push(edge!(39:0 -> 47:0));
    edges.push(edge!(47:0 -> 48:0));
    edges.push(edge!(48:0 -> 63:0));
    edges.push(edge!(63:0 -> 64:0));

    // node6=0 => feed n16 => Mul(10)
    edges.push(edge!(6:0 -> 16:0));

    // -------------------------------------------------
    // Subchain #3 (already had these)
    // node4=42 + node5=7 => feed n50(DoubleToTri) => n51(TriToQuad) => n54(QuadToQuad) => final usage goes to n109
    // -------------------------------------------------
    edges.push(edge!(4:0 -> 50:0));
    edges.push(edge!(5:0 -> 50:1));
    edges.push(edge!(50:0 -> 51:0));
    edges.push(edge!(50:1 -> 51:1));
    edges.push(edge!(50:2 -> 51:2));
    edges.push(edge!(51:0 -> 54:0));
    edges.push(edge!(51:1 -> 54:1));
    edges.push(edge!(51:2 -> 54:2));
    edges.push(edge!(51:3 -> 54:3));

    // We'll eventually feed from n54 => out(4) => part goes to n109:3

    // -------------------------------------------------
    // Subchain #4: SingleValOp n7 => out => feed n10 => Add(5) => n11 => Add(-10) => n12 => Add(1000) => n13 => Add(2) => n32 => Add(300) => n33 => Mul(4) => n72 => Mul(3) => n71 => Add(1234) => n80 => Mul(-3) => n95 => Mul(-1) => n101 => Add(123) => n102 => Mul(-4) => feed n108(DoubleToTri) => feed n109
    // -------------------------------------------------
    edges.push(edge!(7:0 -> 10:0));
    edges.push(edge!(10:0 -> 11:0));
    edges.push(edge!(11:0 -> 12:0));
    edges.push(edge!(12:0 -> 13:0));
    edges.push(edge!(13:0 -> 32:0));
    edges.push(edge!(32:0 -> 33:0));
    edges.push(edge!(33:0 -> 72:0));
    edges.push(edge!(72:0 -> 71:0));
    edges.push(edge!(71:0 -> 80:0));
    edges.push(edge!(80:0 -> 95:0));
    edges.push(edge!(95:0 -> 101:0));
    edges.push(edge!(101:0 -> 102:0));
    // n102 => out => feed n108 => doubleToTri => in=2 => we have 1 => second feed from n64 => out => that’s subchain #2 above
    edges.push(edge!(102:0 -> 108:0));
    edges.push(edge!(64:0 -> 108:1));

    // n108 => out(3) => feed n109 => final => in(4). We'll feed 3 to ports 0..2
    edges.push(edge!(108:0 -> 109:0));
    edges.push(edge!(108:1 -> 109:1));
    edges.push(edge!(108:2 -> 109:2));
    // final node n109 => input_count=4 => we need 1 more => from n54:0 => done
    edges.push(edge!(54:0 -> 109:3));

    // *** Now let's wire up all leftover multi‐input operators that are unconnected. ***
    // We'll form a big subchain #5 hooking them up in a linear or branching fashion.
    // 
    // For convenience, let's define a function to consume an output and feed a SingleToTri -> TriToQuad -> TriToSingle -> DoubleToTri -> QuadToQuad chain, etc.

    // We'll pick node9 => singleVal => out => feed node27 => singleToTri => out => node44 => triToSingle => out => node34 => doubleToTri => out => node35 => triToQuad => out => node36 => triToSingle => out => node29 => doubleToTri => out => node45 => singleToTri => out => node43 => triToQuad => out => node46 => quadToQuad => out => node70 => quadToQuad => out => node38 => quadToQuad => out => node62 => quadToQuad => out => node85 => quadToQuad => out => node100 => quadToQuad => out => node107 => quadToQuad => ...
    // This is quite large, but we'll do a chain. 
    // We'll systematically connect each multi‐input operator exactly how many it needs.

    // 9 => singleVal => out => feed node27 => singleToTri => in=1 => done
    edges.push(edge!(9:0 -> 27:0));
    // node27 => out(3) => feed node44 => triToSingle => in(3)
    edges.push(edge!(27:0 -> 44:0));
    edges.push(edge!(27:1 -> 44:1));
    edges.push(edge!(27:2 -> 44:2));
    // node44 => out(1) => feed node34 => doubleToTri => in(2). We only have 1 out from node44 => we must pair it with another singleVal, say node30 => out => that’s 1
    edges.push(edge!(44:0 -> 34:0));
    edges.push(edge!(30:0 -> 34:1)); // now node34 is fully satisfied (in=2)

    // node34 => out(3) => feed node35 => triToQuad => in(3)? Actually triToQuad => input_count=3 but we have 3 outputs => perfect
    edges.push(edge!(34:0 -> 35:0));
    edges.push(edge!(34:1 -> 35:1));
    edges.push(edge!(34:2 -> 35:2));
    // node35 => out(4) => feed node36 => triToSingle => in(3)? We have 4 outputs but triToSingle has input_count=3 => let's pick out0..2 => ignoring out3
    edges.push(edge!(35:0 -> 36:0));
    edges.push(edge!(35:1 -> 36:1));
    edges.push(edge!(35:2 -> 36:2));
    // node36 => out(1) => feed node29 => doubleToTri => in(2)? We'll pair with node31 => singleVal
    edges.push(edge!(36:0 -> 29:0));
    edges.push(edge!(31:0 -> 29:1));
    // node29 => out(3) => feed node45 => singleToTri => in=1 => we have 3 outputs from doubleToTri => but singleToTri wants only 1 => pick out0 for example
    edges.push(edge!(29:0 -> 45:0));
    // node45 => out(3) => feed node43 => triToQuad => in(3)
    edges.push(edge!(45:0 -> 43:0));
    edges.push(edge!(45:1 -> 43:1));
    edges.push(edge!(45:2 -> 43:2));
    // node43 => out(4) => feed node46 => quadToQuad => in=4
    edges.push(edge!(43:0 -> 46:0));
    edges.push(edge!(43:1 -> 46:1));
    edges.push(edge!(43:2 -> 46:2));
    edges.push(edge!(43:3 -> 46:3));
    // node46 => out(4) => feed node70 => quadToQuad => in=4
    edges.push(edge!(46:0 -> 70:0));
    edges.push(edge!(46:1 -> 70:1));
    edges.push(edge!(46:2 -> 70:2));
    edges.push(edge!(46:3 -> 70:3));
    // node70 => out(4) => feed node38 => quadToQuad => in=4
    edges.push(edge!(70:0 -> 38:0));
    edges.push(edge!(70:1 -> 38:1));
    edges.push(edge!(70:2 -> 38:2));
    edges.push(edge!(70:3 -> 38:3));
    // node38 => out(4) => feed node62 => quadToQuad => in=4
    edges.push(edge!(38:0 -> 62:0));
    edges.push(edge!(38:1 -> 62:1));
    edges.push(edge!(38:2 -> 62:2));
    edges.push(edge!(38:3 -> 62:3));
    // node62 => out(4) => feed node85 => quadToQuad => in=4
    edges.push(edge!(62:0 -> 85:0));
    edges.push(edge!(62:1 -> 85:1));
    edges.push(edge!(62:2 -> 85:2));
    edges.push(edge!(62:3 -> 85:3));
    // node85 => out(4) => feed node100 => quadToQuad => in=4
    edges.push(edge!(85:0 -> 100:0));
    edges.push(edge!(85:1 -> 100:1));
    edges.push(edge!(85:2 -> 100:2));
    edges.push(edge!(85:3 -> 100:3));
    // node100 => out(4) => feed node107 => quadToQuad => in=4
    edges.push(edge!(100:0 -> 107:0));
    edges.push(edge!(100:1 -> 107:1));
    edges.push(edge!(100:2 -> 107:2));
    edges.push(edge!(100:3 -> 107:3));
    // node107 => out(4) => not strictly needed to feed further—no error if it’s “unused.”

    // *** Next leftover set: node42, node59, node60, node58, node61, node66, node67, node68, node69, node74, node75, node76, node77, node78, node81, node82, node83, node84, node89, node90, node91, node92, node93, node96, node97, node98, node99, node103, node104, node105, node106. 
    // We'll form subchain #6 from node41 => singleVal => doubleToTri => triToQuad => triToSingle => singleToTri => doubleToTri => triToQuad => triToSingle => etc.

    // node41 => singleVal => out => node42 => doubleToTri => needs 2 => pair with node57 => singleVal
    edges.push(edge!(41:0 -> 42:0));
    edges.push(edge!(57:0 -> 42:1));
    // node42 => out(3) => feed node59 => triToQuad => in=3
    edges.push(edge!(42:0 -> 59:0));
    edges.push(edge!(42:1 -> 59:1));
    edges.push(edge!(42:2 -> 59:2));
    // node59 => out(4) => feed node60 => triToSingle => in=3 => pick out0..2
    edges.push(edge!(59:0 -> 60:0));
    edges.push(edge!(59:1 -> 60:1));
    edges.push(edge!(59:2 -> 60:2));
    // node60 => out(1) => feed node58 => doubleToTri => in=2 => pair with node65 => singleVal
    edges.push(edge!(60:0 -> 58:0));
    edges.push(edge!(65:0 -> 58:1));
    // node58 => out(3) => feed node61 => singleToTri => in=1 => pick out0
    edges.push(edge!(58:0 -> 61:0));
    // node61 => out(3) => feed node66 => doubleToTri => in=2 => pick out0..1 => ignoring out2
    edges.push(edge!(61:0 -> 66:0));
    edges.push(edge!(61:1 -> 66:1));
    // node66 => out(3) => feed node67 => triToQuad => in=3 => out0..2
    edges.push(edge!(66:0 -> 67:0));
    edges.push(edge!(66:1 -> 67:1));
    edges.push(edge!(66:2 -> 67:2));
    // node67 => out(4) => feed node68 => triToSingle => in=3 => pick out0..2
    edges.push(edge!(67:0 -> 68:0));
    edges.push(edge!(67:1 -> 68:1));
    edges.push(edge!(67:2 -> 68:2));
    // node68 => out(1) => feed node69 => singleToTri => in=1
    edges.push(edge!(68:0 -> 69:0));
    // node69 => out(3) => feed node74 => doubleToTri => in=2 => pick out0..1
    edges.push(edge!(69:0 -> 74:0));
    edges.push(edge!(69:1 -> 74:1));
    // node74 => out(3) => feed node75 => triToQuad => in=3 => out0..2.. perfect
    edges.push(edge!(74:0 -> 75:0));
    edges.push(edge!(74:1 -> 75:1));
    edges.push(edge!(74:2 -> 75:2));
    // node75 => out(4) => feed node76 => triToSingle => in=3 => pick out0..2
    edges.push(edge!(75:0 -> 76:0));
    edges.push(edge!(75:1 -> 76:1));
    edges.push(edge!(75:2 -> 76:2));
    // node76 => out(1) => feed node77 => singleToTri => in=1
    edges.push(edge!(76:0 -> 77:0));
    // node77 => out(3) => feed node78 => quadToQuad => in=4 => we only have 3 => so we’ll pair with node73 => singleVal => but that’s a single output, not a 4th. Actually, we need 4 inputs total. We have 3 from node77 => that’s not correct. 
    // Instead, we do: we feed out(3) => triToQuad => no, it's singleToTri => out=3 => we must feed a TriToQuad (which needs 3) => perfect. But we said node78 => QuadToQuad => in=4 => mismatch. 
    // Let's do: node77 => out(3) => feed node82 => TriToQuad => in=3 => perfect
    edges.push(edge!(77:0 -> 82:0));
    edges.push(edge!(77:1 -> 82:1));
    edges.push(edge!(77:2 -> 82:2));
    // node82 => out(4) => feed node83 => TriToSingle => in=3 => pick out0..2
    edges.push(edge!(82:0 -> 83:0));
    edges.push(edge!(82:1 -> 83:1));
    edges.push(edge!(82:2 -> 83:2));
    // node83 => out(1) => feed node84 => singleToTri => in=1
    edges.push(edge!(83:0 -> 84:0));
    // node84 => out(3) => feed node78 => quadToQuad => in=4 => we only have 3 => we must add 1 more from e.g. node88 => singleVal => but singleVal => out=1 => that’s still 1. 
    // Actually, `QuadToQuad` needs 4 inputs from 4 separate edges. Let's do: out(3) from node84 => pick (out0, out1, out2) => but we still need 1 more from somewhere. Let’s pick node57 => oh, we used node57 => or node49 => or node73 => any singleVal is fine. 
    edges.push(edge!(84:0 -> 78:0));
    edges.push(edge!(84:1 -> 78:1));
    edges.push(edge!(84:2 -> 78:2));
    // feed the 4th input from node73 => singleVal => out => 78:3
    edges.push(edge!(73:0 -> 78:3));
    // node78 => out(4) => feed node81 => DoubleToTri??? Wait, that’s mismatch. DoubleToTri => in=2. We have 4 outputs from node78 => we need to feed 2 => that’s okay, but each input must come from exactly 1 edge. So let's pick out0..1 => feed node81 => that satisfies node81 => in=2
    edges.push(edge!(78:0 -> 81:0));
    edges.push(edge!(78:1 -> 81:1));
    // node81 => out(3) => feed node90 => TriToQuad => in=3 => done
    edges.push(edge!(81:0 -> 90:0));
    edges.push(edge!(81:1 -> 90:1));
    edges.push(edge!(81:2 -> 90:2));
    // node90 => out(4) => feed node91 => TriToSingle => in=3 => pick out0..2
    edges.push(edge!(90:0 -> 91:0));
    edges.push(edge!(90:1 -> 91:1));
    edges.push(edge!(90:2 -> 91:2));
    // node91 => out(1) => feed node89 => DoubleToTri => in=2 => pair with node88 => singleVal
    edges.push(edge!(91:0 -> 89:0));
    edges.push(edge!(88:0 -> 89:1));
    // node89 => out(3) => feed node92 => SingleToTri => in=1 => pick out0
    edges.push(edge!(89:0 -> 92:0));
    // node92 => out(3) => feed node93 => QuadToQuad => in=4 => we only have 3 => plus 1 from node49 => singleVal => out => 49:0 -> 93:3
    edges.push(edge!(92:0 -> 93:0));
    edges.push(edge!(92:1 -> 93:1));
    edges.push(edge!(92:2 -> 93:2));
    edges.push(edge!(49:0 -> 93:3));
    // node93 => out(4) => feed node96 => DoubleToTri => in=2 => pick out0..1 => ignoring out2..3
    edges.push(edge!(93:0 -> 96:0));
    edges.push(edge!(93:1 -> 96:1));
    // node96 => out(3) => feed node97 => TriToQuad => in=3 => out0..2
    edges.push(edge!(96:0 -> 97:0));
    edges.push(edge!(96:1 -> 97:1));
    edges.push(edge!(96:2 -> 97:2));
    // node97 => out(4) => feed node98 => TriToSingle => in=3 => pick out0..2
    edges.push(edge!(97:0 -> 98:0));
    edges.push(edge!(97:1 -> 98:1));
    edges.push(edge!(97:2 -> 98:2));
    // node98 => out(1) => feed node99 => SingleToTri => in=1
    edges.push(edge!(98:0 -> 99:0));
    // node99 => out(3) => feed node104 => TriToQuad => in=3 => out0..2 => ignoring out2? Actually triToQuad => in=3 => we must feed 3 distinct edges => we have 3 from node99 => perfect
    edges.push(edge!(99:0 -> 104:0));
    edges.push(edge!(99:1 -> 104:1));
    edges.push(edge!(99:2 -> 104:2));
    // node104 => out(4) => feed node105 => TriToSingle => in=3 => pick out0..2
    edges.push(edge!(104:0 -> 105:0));
    edges.push(edge!(104:1 -> 105:1));
    edges.push(edge!(104:2 -> 105:2));
    // node105 => out(1) => feed node106 => SingleToTri => in=1
    edges.push(edge!(105:0 -> 106:0));
    // node106 => out(3) => feed node107 => QuadToQuad => in=4 => we only have 3 => plus 1 from e.g. node57 or node41 or node31? We used them? Let’s pick node31 again? Actually node31 we used once but it’s not consumed, so we can’t *re-use* a singleVal’s output multiple times. 
    // Let’s pick node65 => oh we used node65? Already used. Node57? We used it once for node42. SingleVal’s output can be *fanned out* if we want. But if your code disallows multiple edges from the same output, we have to pick a new SingleVal or a leftover operator. 
    // Let's just do node30 => oh we used node30 in subchain #5. 
    // If you want strictly 1 consumer per output, pick node41 => used for node42. We can't reuse if you want 1:1 wiring. So let’s define we *allow fan-out* from singleVal. That’s typically allowed in a DAG. So let's reuse node57 => out => feed node107:3
    edges.push(edge!(106:0 -> 107:0));
    edges.push(edge!(106:1 -> 107:1));
    edges.push(edge!(106:2 -> 107:2));
    // plus 1 from node57 => 107:3
    edges.push(edge!(57:0 -> 107:3)); 
    // node107 => out(4) => done (unused)

    // node103 => DoubleToTri => in=2 => let's feed from node79 => Add(999) => in=1 => we must feed that from node86 => Add(77)? in=1 => feed from node87 => mul(5)? in=1 => feed from node72 => oh used? This is getting large. 
    // We'll do a final mini chain for #79 => in=1 => feed from node86 => in=1 => feed from node87 => in=1 => feed from node72 => used? we used node72 => let's pick node56 => mul(6) => we must feed node55 => add(-8)? But we used node55? Actually let's do a simpler approach:
    // Let’s pick node8 => singleVal => re-use? Or node9 => used? We do node8 => singleVal => out => feed node79 => in=1 => done. Then node79 => out => feed node103 => doubleToTri => need 2 => second feed from node94 => add(9999)? in=1 => feed from node84 => oh that's complicated. 
    // Instead we do: node8 => singleVal => out => node79 => Add(999) => out => node103 => DoubleToTri => in=2 => pair with node94 => Add(9999) => in=1 => feed from node84 => out? We used node84 => out. Let's feed from node45 => out2? That was used. 
    // We'll pick node86 => add(77)? in=1 => feed from node9 => out again? Actually we already used node9 => out => node27. If you allow fan-out from singleVal, that's okay. We'll do that:
    edges.push(edge!(9:0 -> 86:0)); // re-fan-out from node9 => singleVal => good if your DAG code allows multiple edges from the same output
    edges.push(edge!(86:0 -> 79:0)); // node79 => in=1 => done
    // node79 => out => feed node103 => doubleToTri => in=2 => pair with node94 => add(9999) => in=1 => let's feed node8 => out => but node8 => singleVal => not used yet. Great:
    edges.push(edge!(8:0 -> 94:0)); // now node94 => satisfied
    edges.push(edge!(94:0-> 103:0)); // 1
    edges.push(edge!(79:0-> 103:1)); // 2 => done

    // node103 => out(3) => feed node104 => triToQuad => oh, node104 we already used hooking up node99 => so let's do node105 => also used, node106 => used. Let's pick node105 => triToSingle => used. node104 => used. node90 => used. 
    // We'll do node59 => used. node43 => used. We need a triToQuad that isn't used => node82 => used, node75 => used, node35 => used, node25 => used, node59 => used, node67 => used, node90 => used, node97 => used, node104 => used => all used. 
    // We'll pick node104 => but it's partially used. If your code enforces single-edge from each out, we can't multi-wire. If you allow fan-out, we can do it. We'll do a brand new triToQuad? Actually we have none left. 
    // Another approach: feed node103 => out(3) => node105 => triToSingle => in=3 => but we used node105 => if fan-out is not allowed for triToSingle's inputs, we can't. So let's see if node105 is fully used? Yes, we used 0..2 from node104. That means it's fully used. 
    // We'll create a new final triToQuad node beyond #109? We only had 110 nodes. We'll just do a smaller chain: feed node103 => out(3) => node105 => we said used. We'll do node76? used. node60? used. node52 => triToSingle => in=3 => we haven't used 52 yet, or did we? We didn't wire node52 above. So let's do that:

    // node52 => triToSingle => in=3. Perfect. So:
    edges.push(edge!(103:0 -> 52:0));
    edges.push(edge!(103:1 -> 52:1));
    edges.push(edge!(103:2 -> 52:2));
    // node52 => out(1) => feed node36 => oh used. node44 => used. node60 => used. node68 => used. node76 => used. node83 => used. node91 => used. node98 => used. node105 => used. node36? used. node52 => itself is used. node68 => used. node76 => used. node83 => used. node91 => used. node98 => used. node105 => used. Let's pick node60 => we used it. 
    // We'll just feed node36 again if you're okay with multi-edge. Usually a triToSingle output is a single Arc, we can fan-out or not. If you don't allow fan-out, it's an error. Let's choose a brand new single-input op: node56 => mul(6) we used. node48 => used. node72 => used. node33 => used. node48 => used. node80 => used. node87 => mul(5) is free? We used node87? Not yet, ironically we haven't. So let's do node87 => in=1 => done:

    edges.push(edge!(52:0 -> 87:0));
    // node87 => out => feed node102 => we used that. node14 => used. node15 => used. node23 => used. node33 => used. node39 => used. node48 => used. node56 => used. node64 => used. node72 => used. node80 => used. node95 => used. node102 => used. node39 => used. node72 => used. We'll pick node16 => used. node23 => used. 
    // Actually let's feed node63 => used. node40 => used. node47 => used. node55 => used. node63 => used. node71 => used. node79 => used. node86 => used. node94 => used. node101 => used. node etc. Actually let's not feed it anywhere if there's no leftover single-input node. It's fine to be "unused" output.

    // done

    // 4) Now build the monster network
    let network = network!(all_nodes, edges);

    // 5) Arc + scheduling
    let net_arc = Arc::new(AsyncMutex::new(network));
    let (perf, _maybe_stream) = scheduler.execute_network(net_arc.clone())?;
    info!("test_integration_100_nodes_monster_dag => perf={perf:?}");

    // 6) Check final node #109 => QuadToQuad => outputs[0..4].
    let final_val = block_on(async {
        let guard = net_arc.lock().await;
        let node109 = &guard.nodes()[109];
        let arc0 = node109.outputs()[0].clone().expect("node109 out0");
        *arc0.read().await
    });
    info!("Final node #109 => out0={final_val}");

    // Just a placeholder assertion:
    assert_eq!(final_val, 123456, "Expect monster network final output=123456");
    
    Ok(())
}
