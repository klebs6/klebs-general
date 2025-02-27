// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_05_too_many_inputs.rs ]
//! Attempting to define 5 inputs (input0..input4) → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="five_in",
    opcode="OpCode::TestOp",
    input0="i32",
    input1="i32",
    input2="i32",
    input3="i32",
    input4="i32" // this is the 5th → should fail
)]
pub struct TooManyInputs {
    name: String,
}

impl TooManyInputs {
    pub async fn five_in(&self, _x0: i32, _x1: i32, _x2: i32, _x3: i32, _x4: i32) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

