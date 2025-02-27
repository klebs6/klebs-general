// ---------------- [ File: hydro2-basic-operators/src/double_to_tri_op.rs ]
crate::ix!();

// --------------------------------------
// DoubleToTriOp
// --------------------------------------
#[derive(NamedItem,Operator,Debug)]
#[operator(
    execute="double_to_tri",
    opcode="OpCode::DoubleToTriOp",
    input0="T", 
    input1="T", 
    output0="T",
    output1="T",
    output2="T"
)]
pub struct DoubleToTriOp<T> 
where T: Copy + Display + Mul<T,Output=T> + Add<T,Output=T> + Sub<T,Output=T> + Debug + Send + Sync
{
    name: String,
    _0: PhantomData<T>,
}

impl<T> DoubleToTriOp<T> 
where T: Copy + Display + Mul<T,Output=T> + Add<T,Output=T> + Sub<T,Output=T> + Debug + Send + Sync
{
    async fn double_to_tri(&self, input0: &T, input1: &T) -> NetResult<(T,T,T)> {
        let left = *input0;
        let right= *input1;
        let sum = left + right;
        let diff= left - right;
        let prod= left * right;
        info!("DoubleToTriOp => left={}, right={} => sum={}, diff={}, prod={}", left, right, sum, diff, prod);
        Ok((sum,diff,prod))
    }
}

#[cfg(test)]
mod double_to_tri_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_double_to_tri_i32() -> Result<(), NetworkError> {
        let op = DoubleToTriOp::<i32> {
            name: "d2t".into(),
            _0:   Default::default(),
        };
        let input = [
            Some(&DoubleToTriOpIO::Input0(10)),
            Some(&DoubleToTriOpIO::Input1(3)),
            None, None,
        ];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(DoubleToTriOpIO::Output0(13)));
        assert_eq!(out[1], Some(DoubleToTriOpIO::Output1(7)));
        assert_eq!(out[2], Some(DoubleToTriOpIO::Output2(30)));
        Ok(())
    }

    #[tokio::test]
    async fn test_double_to_tri_floats() -> Result<(), NetworkError> {
        let op = DoubleToTriOp::<f64> {
            name: "d2tf64".into(),
            _0:   Default::default(),
        };
        let i0 = DoubleToTriOpIO::Input0(2.5);
        let i1 = DoubleToTriOpIO::Input1(0.5);
        let input = [Some(&i0), Some(&i1), None, None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        // sum=3.0, diff=2.0, product=1.25
        assert_eq!(out[0], Some(DoubleToTriOpIO::Output0(3.0)));
        assert_eq!(out[1], Some(DoubleToTriOpIO::Output1(2.0)));
        assert_eq!(out[2], Some(DoubleToTriOpIO::Output2(1.25)));
        Ok(())
    }
}
