// ---------------- [ File: tests/trybuild/fail_generics_bad_unused_param.rs ]
// tests/trybuild/fail_generics_bad_unused_param.rs

use hydro2_operator::*;         // wherever you define #[derive(Operator)]
use hydro2_operator_derive::*;         // wherever you define #[derive(Operator)]
use named_item::*;
use named_item_derive::*;
use std::fmt::Debug;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="do_nothing",
    opcode="BasicOpCode::TestOp",
    input0="X" // but we won't store X or do anything with it
)]
pub struct UnusedParamOp<X: Send + Sync + Debug> {
    name: String,
    // no fields referencing X
}

impl<X: Send + Sync + Debug> UnusedParamOp<X> {
    async fn do_nothing(&self, _val: &X) -> NetResult<()> {
        // This might or might not use X, but the struct doesn't store it
        Ok(())
    }
}

fn main() {}

use std::sync::Arc;
