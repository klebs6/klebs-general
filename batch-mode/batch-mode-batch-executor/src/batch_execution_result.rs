// ---------------- [ File: src/batch_execution_result.rs ]
crate::ix!();

#[derive(Debug,Getters)]
#[getset(get="pub")]
pub struct BatchExecutionResult {
    outputs: Option<BatchOutputData>,
    errors:  Option<BatchErrorData>,
}

impl BatchExecutionResult {

    pub fn new(outputs: Option<BatchOutputData>, errors: Option<BatchErrorData>) -> Self {
        Self { outputs, errors }
    }
}
