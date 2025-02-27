// ---------------- [ File: hydro2-basic-operators/src/constant_op.rs ]
crate::ix!();

// --------------------------------------
// ConstantOp<T>
// --------------------------------------
#[derive(NamedItem,Operator,Debug)]
#[operator(execute="emit",opcode="BasicOpCode::ConstantOp",output0="T")]
pub struct ConstantOp<T> 
where T: Display + Copy + Debug + Send + Sync + PartialEq,
{
    name:  String,
    value: T,
}

unsafe impl<T> Send for ConstantOp<T> where T: Display + Copy + Debug + Send + Sync + PartialEq {}
unsafe impl<T> Sync for ConstantOp<T> where T: Display + Copy + Debug + Send + Sync + PartialEq {}

impl<T> ConstantOp<T> 
where T: Display + Copy + Debug + Send + Sync + PartialEq
{
    pub fn new(v: T) -> Self {
        Self { 
            name:  format!("ConstantOp({})", v),
            value: v,
        }
    }

    async fn emit(&self) -> NetResult<T> {
        info!("OPERATOR running ConstantOp with value: {}", self.value);
        Ok(self.value)
    }
}

#[cfg(test)]
mod constant_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_constant_op_basic() -> Result<(), NetworkError> {
        let cst = ConstantOp::<i32>::new(42);
        let input: [Option<&ConstantOpIO<i32>>;4] = [None,None,None,None];
        let mut out: [Option<ConstantOpIO<i32>>;4] = [None,None,None,None];
        cst.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(ConstantOpIO::Output0(42)));
        Ok(())
    }

    #[tokio::test]
    async fn test_constant_op_floats() -> Result<(), NetworkError> {
        let cst = ConstantOp::<f64>::new(3.14);
        let input = [None,None,None,None];
        let mut out = [None,None,None,None];
        cst.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(ConstantOpIO::Output0(3.14)));
        Ok(())
    }
}
