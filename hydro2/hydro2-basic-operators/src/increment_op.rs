// ---------------- [ File: hydro2-basic-operators/src/increment_op.rs ]
crate::ix!();

// --------------------------------------
// IncrementOperator
// --------------------------------------
#[derive(Default, NamedItem, Operator, Debug)]
#[operator(
    execute = "inc_all",
    opcode  = "OpCode::IncrementOperator",
    input0  = "i32",
    input1  = "i32",
    input2  = "i32",
    input3  = "i32",
    output0 = "i32",
    output1 = "i32",
    output2 = "i32",
    output3 = "i32"
)]
#[named_item(default_name="Mock Increment Operator")]
pub struct IncrementOperator {
    name: String,
}

impl IncrementOperator {
    async fn inc_all(
        &self,
        in0: &i32, in1: &i32, in2: &i32, in3: &i32
    ) -> NetResult<(i32,i32,i32,i32)>
    {
        info!("OPERATOR running IncrementOperator");
        Ok((in0+1, in1+1, in2+1, in3+1))
    }
}
