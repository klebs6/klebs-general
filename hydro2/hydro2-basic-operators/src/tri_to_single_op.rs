// ---------------- [ File: hydro2-basic-operators/src/tri_to_single_op.rs ]
crate::ix!();

// --------------------------------------
// TriToSingleOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="tri_sum",
    opcode="BasicOpCode::TriToSingleOp",
    input0="i32",
    input1="i32",
    input2="i32",
    output0="i32"
)]
pub struct TriToSingleOp {
    name: String,
}

impl TriToSingleOp {
    pub fn new() -> Self {
        Self { name: "TriToSingleOp".into() }
    }

    async fn tri_sum(&self, a: &i32, b: &i32, c: &i32) -> NetResult<i32> {
        let sum = *a + *b + *c;
        info!("TriToSingleOp => a={}, b={}, c={} => sum={}", a, b, c, sum);
        Ok(sum)
    }
}

#[cfg(test)]
mod tri_to_single_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_tri_to_single_op_basic() -> Result<(), NetworkError> {
        let op = TriToSingleOp::new();
        let i0 = TriToSingleOpIO::Input0(2);
        let i1 = TriToSingleOpIO::Input1(3);
        let i2 = TriToSingleOpIO::Input2(4);
        let input = [Some(&i0), Some(&i1), Some(&i2), None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(TriToSingleOpIO::Output0(9)));
        Ok(())
    }
}

