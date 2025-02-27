// ---------------- [ File: hydro2-basic-operators/src/quad_to_quad_op.rs ]
crate::ix!();

// --------------------------------------
// QuadToQuadOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="quad_math",
    opcode="BasicOpCode::QuadToQuadOp",
    input0="i32",
    input1="i32",
    input2="i32",
    input3="i32",
    output0="i32",
    output1="i32",
    output2="i32",
    output3="i32"
)]
pub struct QuadToQuadOp {
    name: String,
}

impl QuadToQuadOp {
    pub fn new() -> Self {
        Self { name: "QuadToQuadOp".into() }
    }

    async fn quad_math(&self, a: &i32, b: &i32, c: &i32, d: &i32) -> NetResult<(i32,i32,i32,i32)> {
        let sum_all = *a + *b + *c + *d;
        let sum_ab  = *a + *b;
        let sum_cd  = *c + *d;
        let product = *a * *b * *c * *d;
        info!("QuadToQuadOp => a={}, b={}, c={}, d={} => sums/products", a,b,c,d);
        Ok((sum_all, sum_ab, sum_cd, product))
    }
}

#[cfg(test)]
mod quad_to_quad_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_quad_to_quad_op_basic() -> Result<(), NetworkError> {
        let op = QuadToQuadOp::new();
        let i0 = QuadToQuadOpIO::Input0(1);
        let i1 = QuadToQuadOpIO::Input1(2);
        let i2 = QuadToQuadOpIO::Input2(3);
        let i3 = QuadToQuadOpIO::Input3(4);

        let input = [Some(&i0), Some(&i1), Some(&i2), Some(&i3)];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        // sum_all=10, sum_ab=3, sum_cd=7, product=24
        assert_eq!(out[0], Some(QuadToQuadOpIO::Output0(10)));
        assert_eq!(out[1], Some(QuadToQuadOpIO::Output1(3)));
        assert_eq!(out[2], Some(QuadToQuadOpIO::Output2(7)));
        assert_eq!(out[3], Some(QuadToQuadOpIO::Output3(24)));
        Ok(())
    }
}
