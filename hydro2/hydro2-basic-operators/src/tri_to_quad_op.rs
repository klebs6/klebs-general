// ---------------- [ File: hydro2-basic-operators/src/tri_to_quad_op.rs ]
crate::ix!();

// --------------------------------------
// TriToQuadOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="tri_to_quad",
    opcode="OpCode::TriToQuadOp",
    input0="i32",
    input1="i32",
    input2="i32",
    output0="i32",
    output1="i32",
    output2="i32",
    output3="i32"
)]
pub struct TriToQuadOp {
    name: String,
}

impl TriToQuadOp {
    pub fn new() -> Self {
        Self { name: "TriToQuadOp".into() }
    }

    async fn tri_to_quad(&self, a: &i32, b: &i32, c: &i32) -> NetResult<(i32,i32,i32,i32)> {
        let sum = *a + *b + *c;
        let product = *a * *b * *c;
        let mi = a.min(b).min(c);
        let ma = a.max(b).max(c);
        info!("TriToQuadOp => a={},b={},c={} => sum={}, product={}, min={}, max={}", a, b, c, sum, product, mi, ma);
        Ok((sum, product, *mi, *ma))
    }
}

#[cfg(test)]
mod tri_to_quad_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_tri_to_quad_op_basic() -> Result<(), NetworkError> {
        let op = TriToQuadOp::new();
        let i0 = TriToQuadOpIO::Input0(1);
        let i1 = TriToQuadOpIO::Input1(5);
        let i2 = TriToQuadOpIO::Input2(2);

        let input = [Some(&i0), Some(&i1), Some(&i2), None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        // sum=8, product=10, min=1, max=5
        assert_eq!(out[0], Some(TriToQuadOpIO::Output0(8)));
        assert_eq!(out[1], Some(TriToQuadOpIO::Output1(10)));
        assert_eq!(out[2], Some(TriToQuadOpIO::Output2(1)));
        assert_eq!(out[3], Some(TriToQuadOpIO::Output3(5)));
        Ok(())
    }
}

