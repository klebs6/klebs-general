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

#[cfg(test)]
mod reconcile_unprocessed_tests {
    use super::*;
    use std::{
        future::Future,
        pin::Pin,
        fs,
    };

    fn mock_process_output<'a>(
        _triple: &'a BatchFileTriple,
        _workspace: &'a (dyn BatchWorkspaceInterface + 'a),
        _ect: &'a ExpectedContentType,
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> {
        Box::pin(async move {
            debug!("mock_process_output called");
            Ok(())
        })
    }

    fn mock_process_error<'a>(
        _triple: &'a BatchFileTriple,
        _ops: &'a [BatchErrorFileProcessingOperation],
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>> {
        Box::pin(async move {
            debug!("mock_process_error called");
            Ok(())
        })
    }

    const MOCK_PROCESS_OUTPUT: BatchWorkflowProcessOutputFileFn = mock_process_output;
    const MOCK_PROCESS_ERROR:  BatchWorkflowProcessErrorFileFn  = mock_process_error;

    #[traced_test]
    async fn test_reconcile_unprocessed_input_only() {
        // Use a real tokio runtime so that tokio::fs calls won't panic.
        let workspace: Arc<dyn BatchWorkspaceInterface> = BatchWorkspace::new_temp()
            .await
            .expect("expected ephemeral workspace");

        // Build a triple with index=42
        let mut triple = BatchFileTriple::new_for_test_with_workspace(workspace.clone());
        triple.set_index(BatchIndex::from(42u64));

        // Write input file in the workspace location:
        let input_path = workspace.input_filename(triple.index());
        fs::write(&input_path, b"fake input").unwrap();
        triple.set_input_path(Some(input_path.to_string_lossy().to_string().into()));

        // Also provide a metadata file in the correct workspace location.
        let meta_path = workspace.metadata_filename(triple.index());
        fs::write(
            &meta_path,
            r#"{"batch_id":"some_mock_batch_id_for_42","input_file_id":"fake_input_file_id_42"}"#
        ).unwrap();

        // Mock client that eventually finishes with no output/error files
        let client_mock = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        client_mock.configure_inprogress_then_complete_with("some_mock_batch_id_for_42", false, false);

        let client_mock = Arc::new(client_mock) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let ect = ExpectedContentType::JsonLines;
        let result = triple.reconcile_unprocessed(
            client_mock.as_ref(),
            &ect,
            &MOCK_PROCESS_OUTPUT,
            &MOCK_PROCESS_ERROR,
        ).await;

        // Should succeed, since there's just an input, no discovered output or error.
        assert!(result.is_ok(), "Input-only triple with no online files should not fail");
    }
}
