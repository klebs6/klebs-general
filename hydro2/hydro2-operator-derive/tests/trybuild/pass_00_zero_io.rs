// ---------------- [ File: hydro2-operator-derive/tests/trybuild/pass_00_zero_io.rs ]
//! Zero I/O: Should compile successfully.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="noop", 
    opcode="OpCode::Nothing"
)]
pub struct EmptyOperator {
    name: String,
}

impl EmptyOperator {
    // The function named in `execute="noop"` must exist:
    pub async fn noop(&self) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

