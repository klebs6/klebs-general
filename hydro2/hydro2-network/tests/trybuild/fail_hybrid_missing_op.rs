// ---------------- [ File: tests/trybuild/fail_hybrid_missing_op.rs ]
// tests/trybuild/fail_hybrid_missing_op.rs

use std::marker::PhantomData;
use hydro2_network_wire_derive::NetworkWire;
use hydro2_operator::*;
use hydro2_operator_derive::*;
use hydro2_network::*;
use hydro2_3p::*;

// We define an operator FooOperator, but do NOT reference it in #[available_operators] => 
// So the expansions that produce references to FooOperatorIO will fail if we build a node with it.

#[derive(Debug, NamedItem, Operator)]
#[operator(
    execute="exec_foo",
    opcode="BasicOpCode::Foo",
    input0="i32",
    output0="i32"
)]
pub struct FooOperator 
{
    name: String,
}

impl FooOperator {
    pub async fn exec_foo(&self, input: &i32) -> NetResult<i32> {
        Ok(input + 1000)
    }
}

#[derive(NetworkWire)]
#[available_operators(
    // We do NOT mention "FooOperator" here => 
    // if we try to build a network with node!(0 => FooOperator), that might fail type-wire checks
    op="BarOperator" // Suppose we define it somewhere else
)]
pub struct MyNetworkWireC {
    _p: PhantomData<i32>,
}

fn main() {
    // Attempt to build a network with node0=FooOperator, but we never added `op="FooOperator"` 
    // => the expansions for the wire type won't have FooOperatorIO => compile error.
    let _node0 = node!(0 => FooOperator { name: "foo".to_string() });
    // ...
}
