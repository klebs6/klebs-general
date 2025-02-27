// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_09_duplicate_opcode.rs ]
//! Specifying `opcode` twice → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="example_fn",
    opcode="BasicOpCode::TestOp",
    opcode="BasicOpCode::AnotherOp" // should fail
)]
pub struct DuplicateOpcode {
    name: String,
}

impl DuplicateOpcode {
    pub async fn example_fn(&self) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

