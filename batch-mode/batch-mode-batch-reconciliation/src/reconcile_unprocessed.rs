// ---------------- [ File: src/reconcile_unprocessed.rs ]
crate::ix!();

/// Trait describing how a `BatchFileTriple` can be reconciled if unprocessed.
#[async_trait]
pub trait ReconcileUnprocessed<E> {
    async fn reconcile_unprocessed(
        &mut self,
        client:                 &dyn LanguageModelClientInterface<E>,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &BatchWorkflowProcessOutputFileFn,
        process_error_file_fn:  &BatchWorkflowProcessErrorFileFn,
    ) -> Result<(), E>;
}

/* 
   TYPE ALIASES: 
   They define the EXACT function pointer signature we require.  
   Notice that each parameter is `'a`, and the second parameter is 
   `&'a (dyn BatchWorkspaceInterface + 'a)`, not just `&'a dyn BatchWorkspaceInterface`.
*/

pub type BatchWorkflowProcessOutputFileFn = for<'a> fn(
    &'a BatchFileTriple,
    &'a (dyn BatchWorkspaceInterface + 'a),
    &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>>;

pub type BatchWorkflowProcessErrorFileFn = for<'a> fn(
    &'a BatchFileTriple,
    &'a [BatchErrorFileProcessingOperation],
) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>>;
