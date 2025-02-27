// ---------------- [ File: hydro2-basic-operators/src/merge2_op.rs ]
crate::ix!();

// --------------------------------------
// Merge2Op
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="merge_op",
    opcode="BasicOpCode::Merge2Op",
    input0="i32",
    input1="i32",
    output0="i32"
)]
pub struct Merge2Op {
    name: String,
}

impl Default for Merge2Op {
    fn default() -> Self {
        Self { name: "Merge2Op".to_string() }
    }
}

impl Merge2Op {

    async fn merge_op(&self, a: &i32, b: &i32) -> NetResult<i32> {
        let sum = *a + *b;
        info!("Merge2Op => a={}, b={}, sum={}", a, b, sum);
        Ok(sum)
    }
}

#[cfg(test)]
mod merge2_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_merge2_op_basic() -> Result<(), NetworkError> {
        let op = Merge2Op::default();
        let i0 = Merge2OpIO::Input0(12);
        let i1 = Merge2OpIO::Input1(8);
        let input = [Some(&i0), Some(&i1), None, None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(Merge2OpIO::Output0(20)));
        Ok(())
    }

    #[tokio::test]
    async fn test_merge2_op_negatives() -> Result<(), NetworkError> {
        let op = Merge2Op::default();
        let in0 = Merge2OpIO::Input0(-5);
        let in1 = Merge2OpIO::Input1(-10);
        let input = [Some(&in0), Some(&in1), None, None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(Merge2OpIO::Output0(-15)));
        Ok(())
    }
}

