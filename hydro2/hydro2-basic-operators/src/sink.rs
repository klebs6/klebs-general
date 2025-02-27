// ---------------- [ File: src/sink.rs ]
crate::ix!();

// --------------------------------------
// SinkOperator<T>
// --------------------------------------
#[derive(NamedItem, Operator, Debug)]
#[operator(
    execute="sink",
    input0="T",
    input1="T",
    input2="T",
    input3="T",
    opcode="BasicOpCode::Sink"
)]
pub struct SinkOperator<T>
where T: Send + Sync + Debug + Copy
{
    name: String,
    _0: PhantomData<T>,
}

impl<T> SinkOperator<T>
where T: Send + Sync + Debug + Copy
{
    pub fn with_name(x: impl AsRef<str>) -> Self {
        Self {
            name: x.as_ref().to_string(),
            _0: Default::default()
        }
    }

    async fn sink(
        &self, 
        _input0: &T, 
        _input1: &T, 
        _input2: &T, 
        _input3: &T
    ) -> NetResult<()> {
        info!("SinkOperator => does nothing yet");
        Ok(())
    }
}

impl<T> Default for SinkOperator<T>
where T: Send + Sync + Debug + Copy
{
    fn default() -> Self {
        SinkOperator::with_name("default")
    }
}
