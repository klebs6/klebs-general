// ---------------- [ File: hydro2-basic-operators/src/single_channel_passthrough_op.rs ]
crate::ix!();

// --------------------------------------
// SingleChannelPassthroughOperator<T>
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="pass_through",
    opcode="BasicOpCode::SingleChannelPassthrough",
    input0="T",
    output0="T"
)]
pub struct SingleChannelPassthroughOperator<T>
where
    T: Copy + Debug + Send + Sync
{
    name: String,
    _0:   PhantomData<T>,
}

impl<T> SingleChannelPassthroughOperator<T>
where
    T: Copy + Debug + Send + Sync
{
    pub fn with_name(x: impl AsRef<str>) -> Self {
        Self {
            name: x.as_ref().to_string(),
            _0:   Default::default(),
        }
    }

    async fn pass_through(&self, val: &T) -> NetResult<T> {
        info!("SingleChannelPassthrough => pass_through: {:?}", val);
        Ok(*val)
    }
}

impl<T> Default for SingleChannelPassthroughOperator<T>
where
    T: Copy + Debug + Send + Sync
{
    fn default() -> Self {
        Self::with_name("single channel passthrough")
    }
}

#[cfg(test)]
mod single_channel_passthrough_operator_tests {
    use super::*;

    #[tokio::test]
    async fn test_single_channel_passthrough_noop() -> Result<(), NetworkError> {
        // This test was originally called “test_no_op_operator_basic”
        // We'll rename to clarify it's testing pass-through with a certain input
        let nop = SingleChannelPassthroughOperator::<i32>::with_name("test_noop");
        assert_eq!(nop.opcode().val(), BasicOpCode::SingleChannelPassthrough.val());
        assert_eq!(nop.name(), "test_noop");
        assert_eq!(nop.input_count(), 1);
        assert_eq!(nop.output_count(), 1);

        // Instead of passing &123_i32, we must pass &<IO>::Input0(123)
        let input_val = SingleChannelPassthroughOperatorIO::<i32>::Input0(123);
        let input = [Some(&input_val), None, None, None];
        let mut out = [None,None,None,None];

        nop.execute(input, &mut out).await?;
        // The operator passes input straight to Output0
        assert_eq!(out[0], Some(SingleChannelPassthroughOperatorIO::Output0(123)));
        Ok(())
    }

    #[tokio::test]
    async fn test_single_channel_passthrough_basic() -> Result<(), NetworkError> {
        let op = SingleChannelPassthroughOperator::<i32>::with_name("PassX");
        let input = [Some(&SingleChannelPassthroughOperatorIO::Input0(123)), None, None, None];
        let mut out = [None,None,None,None];

        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(SingleChannelPassthroughOperatorIO::Output0(123)));
        Ok(())
    }

    #[tokio::test]
    async fn test_single_channel_passthrough_zero() -> Result<(), NetworkError> {
        let op = SingleChannelPassthroughOperator::<i32>::with_name("PassX");
        let input = [Some(&SingleChannelPassthroughOperatorIO::Input0(0)), None, None, None];
        let mut out = [None,None,None,None];
        op.execute(input, &mut out).await?;
        assert_eq!(out[0], Some(SingleChannelPassthroughOperatorIO::Output0(0)));
        Ok(())
    }
}

