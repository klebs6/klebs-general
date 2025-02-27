// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_08_duplicate_execute.rs ]
//! Specifying `execute` twice → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="fn1",
    execute="fn2", // should fail
    opcode="BasicOpCode::TestOp"
)]
pub struct DuplicateExecute {
    name: String,
}

impl DuplicateExecute {
    pub async fn fn1(&self) -> NetResult<()> {
        Ok(())
    }
    pub async fn fn2(&self) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

