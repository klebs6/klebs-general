// ---------------- [ File: hydro2-basic-operators/src/failing_operator.rs ]
crate::ix!();

// --------------------------------------
// FailingOperator<T>
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(execute="fail_now", opcode="BasicOpCode::FailingOp")]
pub struct FailingOperator
{
    name:   String,
    reason: String,
}

impl FailingOperator
{
    pub fn new(name: impl AsRef<str>, reason: impl AsRef<str>) -> Self {
        Self {
            name:   name.as_ref().to_string(),
            reason: reason.as_ref().to_string(),
        }
    }

    async fn fail_now(&self) -> NetResult<()> {
        Err(NetworkError::OperatorFailed {
            reason: self.reason.clone()
        })
    }
}

impl Default for FailingOperator
{
    fn default() -> Self {
        Self {
            name:   "failing_op".into(),
            reason: "default fail reason".into(),
        }
    }
}

#[cfg(test)]
mod failing_operator_tests {
    use super::*;

    #[tokio::test]
    async fn test_failing_operator_default() -> Result<(), NetworkError> {
        let fail = FailingOperator::default();
        let input: [Option<&FailingOperatorIO>;4] = [None,None,None,None];
        let mut out: [Option<FailingOperatorIO>;4] = [None,None,None,None];

        let res = fail.execute(input, &mut out).await;
        assert!(res.is_err());
        if let Err(NetworkError::OperatorFailed { reason }) = res {
            assert_eq!(reason, "default fail reason");
        } else {
            panic!("expected OperatorFailed");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_failing_operator_custom() -> Result<(), NetworkError> {
        let fail = FailingOperator::new("myfail", "some reason");
        let input = [None, None, None, None];
        let mut out = [None,None,None,None];
        let err = fail.execute(input, &mut out).await.unwrap_err();
        match err {
            NetworkError::OperatorFailed { reason } => {
                assert_eq!(reason, "some reason");
            }
            _ => panic!("Expected OperatorFailed"),
        }
        Ok(())
    }
}
