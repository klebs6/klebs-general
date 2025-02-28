// ---------------- [ File: src/checkpoint_callback.rs ]
crate::ix!();

/// A trait invoked periodically to record checkpoint data.
/// This can write partial progress to disk, a DB, etc.
#[async_trait]
pub trait CheckpointCallback: Debug + Send + Sync {

    async fn checkpoint(&self, completed_nodes: &[usize]) -> Result<(), NetworkError>;
}

#[derive(Debug)]
pub struct NoOpCheckpointCallback;
unsafe impl Send for NoOpCheckpointCallback {}
unsafe impl Sync for NoOpCheckpointCallback {}

#[async_trait]
impl CheckpointCallback for NoOpCheckpointCallback {

    async fn checkpoint(&self, completed_nodes: &[usize]) 
        -> Result<(), NetworkError> 
    {
        Ok(())
    }
}

//-----------------------------------[mock]
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
