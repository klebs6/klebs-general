// ---------------- [ File: src/reconcile_unprocessed.rs ]
crate::ix!();

#[async_trait]
pub trait ReconcileUnprocessed<OutputF,ErrorF,OFut,EFut,E,C> 
where
    OutputF: Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:  Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:    Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:    Future<Output = Result<(), BatchErrorProcessingError>> + Send,
    C:       LanguageModelClientInterface<E>,
{
    async fn reconcile_unprocessed(
        &mut self,
        client:                 &C,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputF,
        process_error_file_fn:  &ErrorF,
    ) -> Result<(), BatchReconciliationError>;
}
