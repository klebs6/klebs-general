// ---------------- [ File: tests/trybuild/pass_02_multi_in_out.rs ]
//! Multiple in/out: Should compile successfully.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="multi_op",
    opcode="BasicOpCode::MultiThing",
    input0="u32",
    input1="i64",
    output0="bool",
    output1="Vec<u8>"
)]
pub struct MultiIOOp {
    name: String,
}

impl MultiIOOp {
    pub async fn multi_op(
        &self, 
        input0: &u32, 
        input1: &i64
    ) -> NetResult<(bool, Vec<u8>)> {
        let output0 = *input0 > 100; 
        let output1 = input1.to_le_bytes().to_vec();
        Ok((output0, output1))
    }
}

fn main() {}

use std::sync::Arc;
