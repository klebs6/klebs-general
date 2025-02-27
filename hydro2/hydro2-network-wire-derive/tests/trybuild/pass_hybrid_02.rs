// ---------------- [ File: tests/trybuild/pass_hybrid_02.rs ]
// tests/trybuild/pass_hybrid_02.rs

use std::marker::PhantomData;
use hydro2_network_wire_derive::*;
use hydro2_operator_derive::*;
use hydro2_operator::*;
use named_item::*;
use named_item_derive::*;
use std::fmt::Debug;
use std::ops::Add;

#[derive(Debug, NamedItem, Operator)]
#[operator(
    execute="exec_add",
    opcode="BasicOpCode::AddOp",
    input0="T",
    output0="T"
)]
pub struct AddOperator<T> 
where T: Copy + Debug + Send + Sync + Add<T,Output=T>,
{
    pub add_val: T,
    name: String,
}
impl<T> AddOperator<T> 
where T: Copy + Debug + Send + Sync + Add<T,Output=T>,
{
    pub async fn exec_add(&self, input: &T) -> NetResult<T>
    where T: std::ops::Add<Output=T> + Copy
    {
        Ok(*input + self.add_val)
    }
}

#[derive(Debug,Clone,NetworkWire)]
#[available_operators(
    op="AddOperator<T>"
)]
pub struct MyNetworkWireB<T> 
where T: Copy + Debug + Send + Sync + Add<T,Output=T>
{
    _p: PhantomData<T>,
}

fn main() {
    // This should compile, generating MyNetworkWireBIO<T> with variant AddOperatorIO<T>.
    // If you define more operators, you can list them too.
}
use std::sync::Arc;
