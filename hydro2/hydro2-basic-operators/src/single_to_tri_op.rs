// ---------------- [ File: hydro2-basic-operators/src/single_to_tri_op.rs ]
crate::ix!();

// --------------------------------------
// SingleToTriOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="run_op",
    opcode="BasicOpCode::SingleToTriOp",
    input0="i32",
    output0="i32",
    output1="i32",
    output2="i32"
)]
pub struct SingleToTriOp {
    name: String,
}

impl SingleToTriOp {
    pub fn new() -> Self {
        Self { name: "SingleToTriOp".to_string() }
    }

    async fn run_op(&self, input0: &i32) -> NetResult<(i32, i32, i32)> {
        let val = *input0;
        let out0 = val;
        let out1 = val + 10;
        let out2 = val + 100;
        info!("SingleToTriOp => in={}, out0={}, out1={}, out2={}", val, out0, out1, out2);
        Ok((out0, out1, out2))
    }
}

#[cfg(test)]
mod single_to_tri_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_single_to_tri_op_basic() -> Result<(), NetworkError> {
        let op = SingleToTriOp::new();
        let input_val = SingleToTriOpIO::Input0(42);
        let input = [Some(&input_val), None, None, None];
        let mut out = [None, None, None, None];

        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(SingleToTriOpIO::Output0(42)));
        assert_eq!(out[1], Some(SingleToTriOpIO::Output1(52)));
        assert_eq!(out[2], Some(SingleToTriOpIO::Output2(142)));
        Ok(())
    }

    #[tokio::test]
    async fn test_single_to_tri_op_zero() -> Result<(), NetworkError> {
        let op = SingleToTriOp::new();
        let input0 = SingleToTriOpIO::Input0(0);
        let input = [Some(&input0), None, None, None];
        let mut out = [None, None, None, None];

        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(SingleToTriOpIO::Output0(0)));
        assert_eq!(out[1], Some(SingleToTriOpIO::Output1(10)));
        assert_eq!(out[2], Some(SingleToTriOpIO::Output2(100)));
        Ok(())
    }

    #[tokio::test]
    async fn test_single_to_tri_op_multiple() -> Result<(), NetworkError> {
        let op = SingleToTriOp::new();
        for x in [1, 2, 50, 100] {
            let i0 = SingleToTriOpIO::Input0(x);
            let input = [Some(&i0), None, None, None];
            let mut out = [None, None, None, None];

            op.execute(input, &mut out).await?;
            assert_eq!(out[0], Some(SingleToTriOpIO::Output0(x)));
            assert_eq!(out[1], Some(SingleToTriOpIO::Output1(x+10)));
            assert_eq!(out[2], Some(SingleToTriOpIO::Output2(x+100)));
        }
        Ok(())
    }
}

