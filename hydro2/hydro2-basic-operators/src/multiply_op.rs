// ---------------- [ File: hydro2-basic-operators/src/multiply_op.rs ]
crate::ix!();

// --------------------------------------
// MultiplyOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="multiplier",
    opcode="OpCode::MultiplyOp",
    input0="i32",
    output0="i32"
)]
pub struct MultiplyOp {
    name: String,
    factor: i32,
}

impl MultiplyOp {
    pub fn new(f: i32) -> Self {
        Self {
            name: format!("MultiplyOp(x{})", f),
            factor: f,
        }
    }

    async fn multiplier(&self, input0: &i32) -> NetResult<i32> {
        info!("MultiplyOp => in={}, factor={}", input0, self.factor);
        Ok(*input0 * self.factor)
    }
}

#[cfg(test)]
mod multiply_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiply_op_basic() -> Result<(), NetworkError> {
        let mul = MultiplyOp::new(3);
        let i0 = MultiplyOpIO::Input0(10);
        let input = [Some(&i0), None, None, None];
        let mut out = [None,None,None,None];
        mul.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(MultiplyOpIO::Output0(30)));
        Ok(())
    }

    #[tokio::test]
    async fn test_multiply_op_zero() -> Result<(), NetworkError> {
        let mul = MultiplyOp::new(0);
        let i0 = MultiplyOpIO::Input0(999);
        let input = [Some(&i0), None, None, None];
        let mut out = [None,None,None,None];
        mul.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(MultiplyOpIO::Output0(0)));
        Ok(())
    }

    #[tokio::test]
    async fn test_multiply_op_negative() -> Result<(), NetworkError> {
        let mul = MultiplyOp::new(-2);
        let i0 = MultiplyOpIO::Input0(10);
        let input = [Some(&i0), None, None, None];
        let mut out = [None,None,None,None];
        mul.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(MultiplyOpIO::Output0(-20)));
        Ok(())
    }
}

