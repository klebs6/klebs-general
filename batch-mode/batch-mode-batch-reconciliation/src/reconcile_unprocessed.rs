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
        let mut triple = BatchFileTriple::new_for_test_empty();
        triple.set_index(BatchIndex::from(42u64));
        triple.set_input_path(Some("input_only.json".into()));

        fs::write("input_only.json", b"fake input").unwrap();
        fs::write(
            "mock_metadata_42.json",
            r#"{"batch_id":"some_mock_batch_id_for_42","input_file_id":"fake_input_file_id_42"}"#
        ).unwrap();

        let client_mock = MockLanguageModelClientBuilder::<MockBatchClientError>::default().build().unwrap();

        // NEW: Tell the mock to flip from InProgress -> Completed with no files:
        client_mock.configure_inprogress_then_complete_with("some_mock_batch_id_for_42", false, false);

        let client_mock = Arc::new(client_mock) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let ect = ExpectedContentType::JsonLines;
        let result = triple.reconcile_unprocessed(
            client_mock.as_ref(),
            &ect,
            &MOCK_PROCESS_OUTPUT,
            &MOCK_PROCESS_ERROR,
        ).await;

        assert!(result.is_ok(), "Input-only triple with no online files should not fail");
    }

    #[traced_test]
    fn test_reconcile_unprocessed_input_error_but_mock_processing_fails_action() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(BadWorkspace);
            let mut triple = BatchFileTriple::new_for_test_with_workspace(workspace);

            triple.set_input_path(Some("input.json".into()));
            triple.set_error_path(Some("error.json".into()));

            // We'll create these files so the rename step is what fails, not "not found"
            fs::write("input.json", b"fake input").unwrap();
            fs::write("error.json", b"fake error").unwrap();

            // Also create a metadata file so "check_for_and_download_output_and_error_online"
            // won't fail if it tries that:
            fs::write(
                "mock_metadata_9999.json",
                r#"{"batch_id":"some_mock_batch_id","input_file_id":"fake_input_file_id_9999"}"#
            ).unwrap();

            let client_mock = Arc::new(
                MockLanguageModelClientBuilder::<MockBatchClientError>::default()
                    .build()
                    .unwrap(),
            ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

            let ect = ExpectedContentType::JsonLines;

            let result = triple.reconcile_unprocessed(
                client_mock.as_ref(),
                &ect,
                &MOCK_PROCESS_OUTPUT,
                &MOCK_PROCESS_ERROR,
            ).await;

            assert!(result.is_err(), "We expect an I/O failure with BadWorkspace");
            match result.err().unwrap() {
                MockBatchClientError::BatchReconciliationError { index } => {
                    pretty_assert_eq!(index, *triple.index());
                },
                other => panic!("Unexpected error variant: {:?}", other),
            }
        });
    }
}
