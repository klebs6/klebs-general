// ---------------- [ File: src/full_batch_workspace_interface.rs ]
crate::ix!();

#[async_trait]
pub trait FullBatchWorkspaceInterface<E>:
    BatchWorkspaceInterface
    + LocateBatchFiles<Error=E>
    + FindExistingBatchFileIndices<Error=E>
    + GatherAllBatchTriples<Error=E>
    + Send
    + Sync
{
}

#[async_trait]
impl<T,E> FullBatchWorkspaceInterface<E> for T
where
    T: BatchWorkspaceInterface
        + LocateBatchFiles<Error=E>
        + FindExistingBatchFileIndices<Error=E>
        + GatherAllBatchTriples<Error=E>
        + Send
        + Sync
{
}

#[async_trait]
pub trait GatherAllBatchTriples: Send + Sync {
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

pub trait GetBatchWorkspace<E> {
    fn workspace(&self) -> Arc<dyn FullBatchWorkspaceInterface<E>>;
}
