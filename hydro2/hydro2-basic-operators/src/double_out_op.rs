// ---------------- [ File: hydro2-basic-operators/src/double_out_op.rs ]
crate::ix!();

// --------------------------------------
// DoubleOutOp
// --------------------------------------
#[derive(NamedItem, Debug, Operator)]
#[operator(
    execute="double_out", 
    opcode="OpCode::DoubleOutOp", 
    input0="i32", 
    output0="i32", 
    output1="i32"
)]
pub struct DoubleOutOp {
    name: String,
}

impl Default for DoubleOutOp {
    fn default() -> Self {
        Self {
            name: format!("DoubleOutOp.default"),
        }
    }
}

impl DoubleOutOp {
    async fn double_out(&self, input0: &i32) -> NetResult<(i32, i32)> {
        let val = *input0;
        let out0 = val;
        let out1 = val + 100;
        info!("DoubleOutOp => in={}, out0={}, out1={}", val, out0, out1);
        Ok((out0, out1))
    }
}

#[cfg(test)]
mod double_out_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_double_out_simple() -> Result<(), NetworkError> {
        let op = DoubleOutOp { name: "DoubleOut".to_string() };
        let i0 = DoubleOutOpIO::Input0(5);
        let input_arr = [Some(&i0), None, None, None];
        let mut out_arr = [None, None, None, None];

        op.execute(input_arr, &mut out_arr).await?;
        assert_eq!(out_arr[0], Some(DoubleOutOpIO::Output0(5)));
        assert_eq!(out_arr[1], Some(DoubleOutOpIO::Output1(105)));
        Ok(())
    }

    #[tokio::test]
    async fn test_double_out_negative() -> Result<(), NetworkError> {
        let op = DoubleOutOp { name: "DoubleOut".to_string() };
        let input = [Some(&DoubleOutOpIO::Input0(-10)), None, None, None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(DoubleOutOpIO::Output0(-10)));
        assert_eq!(out[1], Some(DoubleOutOpIO::Output1(90)));
        Ok(())
    }

    #[tokio::test]
    async fn test_double_out_multiple_calls() -> Result<(), NetworkError> {
        let op = DoubleOutOp { name: "DoubleOut".to_string() };
        for &val in &[0, 1, 100] {
            let i0 = DoubleOutOpIO::Input0(val);
            let input = [Some(&i0), None, None, None];
            let mut out = [None,None,None,None];
            op.execute(input, &mut out).await?;
            assert_eq!(out[0], Some(DoubleOutOpIO::Output0(val)));
            assert_eq!(out[1], Some(DoubleOutOpIO::Output1(val+100)));
        }
        Ok(())
    }
}
