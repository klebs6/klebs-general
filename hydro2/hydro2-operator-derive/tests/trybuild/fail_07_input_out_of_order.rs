// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_07_input_out_of_order.rs ]
//! input0, then input2 with no input1 → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="out_of_order",
    opcode="OpCode::TestOp",
    input0="i32",
    input2="i64" // skipping input1 → should fail
)]
pub struct InputOutOfOrder {
    name: String,
}

impl InputOutOfOrder {
    pub async fn out_of_order(&self, _x0: i32, _x1: i64) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

