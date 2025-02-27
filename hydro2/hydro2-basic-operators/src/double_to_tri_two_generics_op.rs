// ---------------- [ File: hydro2-basic-operators/src/double_to_tri_two_generics_op.rs ]
crate::ix!();

// --------------------------------------
// DoubleToTriTwoGenericsOp
// --------------------------------------
#[derive(NamedItem,Operator,Debug)]
#[operator(
    execute="double_to_tri",
    opcode="OpCode::DoubleToTriTwoGenericsOp",
    input0="T", 
    input1="U", 
    input2="i32", 
    output0="T",
    output1="U",
    output2="U"
)]
pub struct DoubleToTriTwoGenericsOp<T,U> 
where T: Copy + Display + Mul<T,Output=T> + Add<T,Output=T> + Sub<T,Output=T> + Debug + Send + Sync,
      U: Copy + Display + Mul<U,Output=U> + Add<U,Output=U> + Sub<U,Output=U> + Debug + Send + Sync,
{
    name: String,
    _0: PhantomData<T>,
    _1: PhantomData<U>,
}

impl<T,U> DoubleToTriTwoGenericsOp<T,U> 
where T: Copy + Display + Mul<T,Output=T> + Add<T,Output=T> + Sub<T,Output=T> + Debug + Send + Sync,
      U: Copy + Display + Mul<U,Output=U> + Add<U,Output=U> + Sub<U,Output=U> + Debug + Send + Sync,
{
    async fn double_to_tri(&self, input0: &T, input1: &U, input2: &i32) -> NetResult<(T,U,U)> {
        let t = *input0;
        let u = *input1;
        let t2 = t * t;
        let u2 = u * u;
        let u3 = u * u * u;
        info!("DoubleToTriTwoGenericsOp => input2={}, t={}, u={} => t2={}, u2={}, u3={}", input2, t, u, t2, u2, u3);
        Ok((t2,u2,u3))
    }
}

#[cfg(test)]
mod double_to_tri_two_generics_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_double_to_tri_two_generics_i32() -> Result<(), NetworkError> {
        let op = DoubleToTriTwoGenericsOp::<i32,f32> {
            name: "d2t".into(),
            _0:   Default::default(),
            _1:   Default::default(),
        };
        let input = [
            Some(&DoubleToTriTwoGenericsOpIO::Input0(10)),
            Some(&DoubleToTriTwoGenericsOpIO::Input1(3.0)),
            Some(&DoubleToTriTwoGenericsOpIO::Input2(3)),
            None,
        ];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(DoubleToTriTwoGenericsOpIO::Output0(100)));
        assert_eq!(out[1], Some(DoubleToTriTwoGenericsOpIO::Output1(9.0)));
        assert_eq!(out[2], Some(DoubleToTriTwoGenericsOpIO::Output2(27.0)));
        Ok(())
    }

    #[tokio::test]
    async fn test_double_to_tri_two_generics_floats() -> Result<(), NetworkError> {
        let op = DoubleToTriTwoGenericsOp::<f64,i32> {
            name: "d2tf64".into(),
            _0:   Default::default(),
            _1:   Default::default(),
        };
        let i0 = DoubleToTriTwoGenericsOpIO::Input0(2.0);
        let i1 = DoubleToTriTwoGenericsOpIO::Input1(5);
        let i2 = DoubleToTriTwoGenericsOpIO::Input2(50);
        let input = [
            Some(&i0), 
            Some(&i1), 
            Some(&i2), 
            None
        ];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        // sum=3.0, diff=2.0, product=1.25
        assert_eq!(out[0], Some(DoubleToTriTwoGenericsOpIO::Output0(4.0)));
        assert_eq!(out[1], Some(DoubleToTriTwoGenericsOpIO::Output1(25)));
        assert_eq!(out[2], Some(DoubleToTriTwoGenericsOpIO::Output2(125)));
        Ok(())
    }
}
