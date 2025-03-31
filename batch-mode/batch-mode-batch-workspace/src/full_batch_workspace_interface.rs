// ---------------- [ File: batch-mode-batch-workspace/src/full_batch_workspace_interface.rs ]
crate::ix!();

#[async_trait]
pub trait FullBatchWorkspaceInterface<E,I>:
    BatchWorkspaceInterface
    + CalculateUnseenInputs<I>
    + LocateBatchFiles<Error=E>
    + FindExistingBatchFileIndices<Error=E>
    + GatherAllBatchTriples<Error=E>
    + Send
    + Sync
    where I: GetTargetPathForAIExpansion + Clone + Debug + Display + Named,
{
}

#[async_trait]
impl<T,E,I> FullBatchWorkspaceInterface<E,I> for T
where
    T: BatchWorkspaceInterface
        + CalculateUnseenInputs<I>
        + LocateBatchFiles<Error=E>
        + FindExistingBatchFileIndices<Error=E>
        + GatherAllBatchTriples<Error=E>
        + Send
        + Sync,
    I: GetTargetPathForAIExpansion + Clone + Debug + Display + Named,
{
}

#[async_trait]
pub trait GatherAllBatchTriples: Send + Sync 
{
    type Error;
    async fn gather_all_batch_triples(
        self: Arc<Self>,
    ) -> Result<Vec<BatchFileTriple>, Self::Error>;
}

#[async_trait]
pub trait FindExistingBatchFileIndices: Send + Sync {
    type Error;
    async fn find_existing_batch_file_indices(
        self: Arc<Self>,
    ) -> Result<HashSet<BatchIndex>, Self::Error>;
}

#[async_trait]
pub trait LocateBatchFiles: Send + Sync {
    type Error;
    async fn locate_batch_files(
        self: Arc<Self>,
        index: &BatchIndex
    ) -> Result<Option<BatchFileTriple>, Self::Error>;
}

pub trait GetBatchWorkspace<E,I> 
    where I: GetTargetPathForAIExpansion + Clone + Debug + Display + Named,
{
    fn workspace(&self) -> Arc<dyn FullBatchWorkspaceInterface<E,I>>;
}
