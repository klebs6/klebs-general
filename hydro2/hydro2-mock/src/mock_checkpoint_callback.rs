// ---------------- [ File: src/mock_checkpoint_callback.rs ]
crate::ix!();

pub type MockCheckpointType = Arc<AsyncMutex<Vec<Vec<usize>>>>;

/// A mock checkpoint callback that records each checkpoint invocation.
#[derive(Builder,Getters,Debug)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct MockCheckpointCallback {
    checkpoints: MockCheckpointType,
}

impl From<&MockCheckpointType> for MockCheckpointCallback {
    fn from(x: &MockCheckpointType) -> Self {
        Self { checkpoints: x.clone() }
    }
}

#[async_trait]
impl CheckpointCallback for MockCheckpointCallback {
    async fn checkpoint(&self, completed_nodes: &[usize]) -> Result<(), NetworkError> {
        let mut guard = self.checkpoints.lock().await;
        guard.push(completed_nodes.to_vec());
        Ok(())
    }
}
