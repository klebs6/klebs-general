// ---------------- [ File: hydro2-basic-operators/src/stream_test_op.rs ]
crate::ix!();

// --------------------------------------
// StreamyOperator
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="run_stream",
    opcode="BasicOpCode::StreamTestOp",
    output0="T",
    output1="T",
    output2="T",
    output3="T"
)]
pub struct StreamyOperator<T> 
where T: PartialEq + Eq + Send + Sync + Debug + Copy + Zero,
{
    name: String,
    outs: [T; 4],
}

impl<T> StreamyOperator<T> 
where T: PartialEq + Eq + Send + Sync + Debug + Copy + Zero,
{
    pub fn new(label: &str) -> Self {
        Self {
            name: label.to_string(),
            outs: [T::zero(),T::zero(),T::zero(),T::zero()],
        }
    }
    pub fn new_with(label: &str, outs: [T;4]) -> Self {
        Self {
            name: label.to_string(),
            outs
        }
    }

    async fn run_stream(&self) -> NetResult<(T,T,T,T)> {
        info!("StreamyOperator '{}' => run_stream", self.name);
        Ok((self.outs[0], self.outs[1], self.outs[2], self.outs[3]))
    }
}

#[cfg(test)]
mod streamy_operator_tests {
    use super::*;

    #[tokio::test]
    async fn test_streamy_operator_default_val() -> Result<(), NetworkError> {
        let s = StreamyOperator::<i32>::new_with("tester",[7,0,0,0]);
        assert_eq!(s.opcode().val(), BasicOpCode::StreamTestOp.val());
        assert_eq!(s.name(), "tester");

        let dummy_input = [None, None, None, None];
        let mut out = [None,None,None,None];
        s.execute(dummy_input,&mut out).await?;
        // Because the operator’s output is [7,0,0,0]
        // we must compare with Some(StreamyOperatorIO::Output0(7)), etc.
        assert_eq!(out[0], Some(StreamyOperatorIO::Output0(7)));
        assert_eq!(out[1], Some(StreamyOperatorIO::Output1(0)));
        assert_eq!(out[2], Some(StreamyOperatorIO::Output2(0)));
        assert_eq!(out[3], Some(StreamyOperatorIO::Output3(0)));
        Ok(())
    }

    #[tokio::test]
    async fn test_streamy_operator_custom_val() -> Result<(), NetworkError> {
        let mut s = StreamyOperator::<i32>::new("my-stream");
        s.outs = [123, 0, 0, 0];
        let dummy_input = [None, None, None, None];
        let mut out = [None,None,None,None];
        s.execute(dummy_input, &mut out).await?;
        assert_eq!(out[0], Some(StreamyOperatorIO::Output0(123)));
        assert_eq!(out[1], Some(StreamyOperatorIO::Output1(0)));
        assert_eq!(out[2], Some(StreamyOperatorIO::Output2(0)));
        assert_eq!(out[3], Some(StreamyOperatorIO::Output3(0)));
        Ok(())
    }

    #[tokio::test]
    async fn test_streamy_operator_basic() -> Result<(), NetworkError> {
        let op = StreamyOperator::<i32>::new_with("tester", [10,20,30,40]);
        let input: [Option<&StreamyOperatorIO<i32>>;4] = [None,None,None,None];
        let mut out: [Option<StreamyOperatorIO<i32>>;4] = [None,None,None,None];
        op.execute(input, &mut out).await?;

        // Expect out0=10, out1=20, out2=30, out3=40
        assert_eq!(out[0], Some(StreamyOperatorIO::Output0(10)));
        assert_eq!(out[1], Some(StreamyOperatorIO::Output1(20)));
        assert_eq!(out[2], Some(StreamyOperatorIO::Output2(30)));
        assert_eq!(out[3], Some(StreamyOperatorIO::Output3(40)));
        Ok(())
    }
}

