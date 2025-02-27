// ---------------- [ File: tests/trybuild/fail_03_missing_opcode.rs ]
//! Missing `opcode` in the `#[operator(...)]` attribute → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="some_fn"
    // opcode is missing → should fail
)]
pub struct MissingOpcodeOp {
    name: String,
}

impl MissingOpcodeOp {

    pub async fn some_fn(&self) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}
