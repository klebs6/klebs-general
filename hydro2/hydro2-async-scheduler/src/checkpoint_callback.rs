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
