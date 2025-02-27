// ---------------- [ File: tests/trybuild/pass_01_single_in_out.rs ]
//! Single in/out: Should compile successfully.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="do_something",
    opcode="BasicOpCode::TestOp",
    input0="bool",
    output0="String"
)]
pub struct SingleIO {
    name: String,
}

impl SingleIO {
    pub async fn do_something(&self, input0: &bool) -> NetResult<String> {
        Ok(format!("Got input: {}", input0))
    }
}

fn main() {}

use std::sync::Arc;
