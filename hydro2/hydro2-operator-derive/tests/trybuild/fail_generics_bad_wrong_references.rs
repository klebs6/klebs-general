// ---------------- [ File: tests/trybuild/fail_generics_bad_wrong_references.rs ]
// tests/trybuild/fail_generics_bad_wrong_references.rs

use hydro2_operator::*;         // wherever you define #[derive(Operator)]
use hydro2_operator_derive::*;         // wherever you define #[derive(Operator)]
use named_item::*;
use named_item_derive::*;
use std::fmt::Debug;

type SomeType = i32;

#[derive(Debug,NamedItem,Operator)]
#[operator(
    execute="some_op",
    opcode="BasicOpCode::TestOp",
    input0="SomeType", output0="SomeType"
)]
pub struct MissingGeneric<S: Copy + Debug + Send + Sync> {
    // We'll store S in a field
    pub data: S,
    name: String,
}

impl<S: Copy + Debug + Send + Sync> MissingGeneric<S> {
    async fn some_op(&self, _val: &S) -> NetResult<S> {
        Ok(self.data)
    }
}

// Then we attempt to reference `MissingGeneric` in a way that lacks <S>:
fn main() {
    // A code snippet that references the macro expansions incorrectly:
    let _ = MissingGeneric {
        data: 10, // forced to i32
        name: "bad".to_string()
    };
    // The generated code might do something like `impl<S> OperatorInterface<MissingGenericIO> for MissingGeneric<S>` 
    // if the macro incorrectly omits <S> in the enum or impl.
}

use std::sync::Arc;
