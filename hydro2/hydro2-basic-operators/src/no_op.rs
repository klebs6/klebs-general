// ---------------- [ File: hydro2-basic-operators/src/no_op.rs ]
crate::ix!();

// --------------------------------------
// NoOpOperator<T>
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="do_nothing",
    opcode="BasicOpCode::NoOp"
)]
pub struct NoOpOperator
{
    name: String,
}

impl NoOpOperator
{
    pub fn with_name(x: impl AsRef<str>) -> Self {
        Self {
            name: x.as_ref().to_string(),
        }
    }

    async fn do_nothing(&self) -> NetResult<()> {
        info!("NoOpOperator => does nothing");
        Ok(())
    }
}

impl Default for NoOpOperator
{
    fn default() -> Self {
        NoOpOperator::with_name("default")
    }
}

#[cfg(test)]
mod no_op_operator_tests {
    use super::*;

    #[tokio::test]
    async fn test_no_op_operator_basic() -> Result<(), NetworkError> {
        let nop = NoOpOperator::default();
        let input: [Option<&NoOpOperatorIO>;4] = [None,None,None,None];
        let mut out: [Option<NoOpOperatorIO>;4] = [None,None,None,None];

        nop.execute(input, &mut out).await?;
        assert_eq!(out[0], None);
        assert_eq!(nop.name(), "default");
        Ok(())
    }

    #[tokio::test]
    async fn test_no_op_operator_with_name() -> Result<(), NetworkError> {
        let nop = NoOpOperator::with_name("my-noop");
        assert_eq!(nop.name(), "my-noop");
        let input = [None,None,None,None];
        let mut out = [None,None,None,None];
        nop.execute(input, &mut out).await?;
        assert_eq!(out, [None,None,None,None]);
        Ok(())
    }
}

