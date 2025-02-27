// ---------------- [ File: src/split_and_double_op.rs ]
crate::ix!();

// --------------------------------------
// SplitAndDoubleOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="split_and_double",
    opcode="BasicOpCode::SplitAndDoubleOp",
    input0="i32",
    output0="i32",
    output1="i32"
)]
pub struct SplitAndDoubleOp {
    name: String,
}

impl Default for SplitAndDoubleOp {
    fn default() -> Self {
        Self { name: "SplitAndDoubleOp".into() }
    }
}

impl SplitAndDoubleOp {

    async fn split_and_double(&self, val: &i32) -> NetResult<(i32,i32)> {
        let out0 = *val;
        let out1 = *val * 2;
        Ok((out0, out1))
    }
}

#[cfg(test)]
mod split_and_double_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_split_and_double() -> Result<(), NetworkError> {
        let op = SplitAndDoubleOp::default();
        let i0 = SplitAndDoubleOpIO::Input0(10);
        let input = [Some(&i0), None, None, None];
        let mut out = [None, None, None, None];
        op.execute(input, &mut out).await?;

        assert_eq!(out[0], Some(SplitAndDoubleOpIO::Output0(10)));
        assert_eq!(out[1], Some(SplitAndDoubleOpIO::Output1(20)));
        Ok(())
    }
}
