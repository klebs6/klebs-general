// ---------------- [ File: hydro2-basic-operators/src/single_value_test_op.rs ]
crate::ix!();

// --------------------------------------
// SingleValOp
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="produce_val",
    opcode="OpCode::SingleValueTestOp",
    output0="i32"
)]
pub struct SingleValOp {
    name: String,
    val:  i32,
}

impl Default for SingleValOp {

    fn default() -> Self {
        Self {
            name: "SingleValOp".to_string(),
            val:  777,
        }
    }
}

impl SingleValOp {

    async fn produce_val(&self) -> NetResult<i32> {
        info!("SingleValOp => produce {}", self.val);
        Ok(self.val)
    }
}

#[cfg(test)]
mod single_val_op_tests {
    use super::*;

    #[tokio::test]
    async fn test_single_val_op_produces_777() -> Result<(), NetworkError> {
        let op = SingleValOp::default();
        let input: [Option<&SingleValOpIO>;4] = [None,None,None,None];
        let mut out: [Option<SingleValOpIO>;4] = [None,None,None,None];

        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(SingleValOpIO::Output0(777)));
        Ok(())
    }
}

