// ---------------- [ File: hydro2-operator-derive/tests/trybuild/fail_04_missing_execute.rs ]
//! Missing `execute` in the `#[operator(...)]` attribute → compile_fail.
use hydro2_operator_derive::*;
use hydro2_operator::*; 
use named_item_derive::*;
use named_item::*;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    // no "execute=..." → should fail
    opcode="OpCode::TestOp"
)]
pub struct MissingExecuteOp {
    name: String,
}

impl MissingExecuteOp {
    // The macro won't know what function is "the one" to call if we never say `execute="..."`.
    pub async fn some_fn(&self) -> NetResult<()> {
        Ok(())
    }
}

fn main() {}

