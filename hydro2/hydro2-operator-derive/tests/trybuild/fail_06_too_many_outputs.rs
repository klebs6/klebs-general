// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_06_too_many_outputs.rs ]
//! Attempting to define output4 → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="five_out",
    opcode="BasicOpCode::TestOp",
    output0="i32",
    output1="i32",
    output2="i32",
    output3="i32",
    output4="i32" // this is the 5th → should fail
)]
pub struct TooManyOutputs {
    name: String,
}

impl TooManyOutputs {
    // The macro expects at most four outputs, but we requested five.
    pub async fn five_out(&self) -> NetResult<(i32,i32,i32,i32,i32)> {
        Ok((1,2,3,4,5))
    }
}

fn main() {}

