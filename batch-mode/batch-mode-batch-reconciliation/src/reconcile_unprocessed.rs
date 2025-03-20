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

    fn mock_process_output_fn<'a>(
        triple: &'a BatchFileTriple,
        workspace: &'a (dyn BatchWorkspaceInterface + 'a),
        ect: &'a ExpectedContentType,
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> {
        // no-op
        Box::pin(async move {
            debug!("mock_process_output_fn called for triple={:?}, workspace=?, ect={:?}", triple.index(), ect);
            Ok(())
        })
    }

    fn mock_process_error_fn<'a>(
        triple: &'a BatchFileTriple,
        ops: &'a [BatchErrorFileProcessingOperation],
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>> {
        // no-op
        Box::pin(async move {
            debug!("mock_process_error_fn called for triple={:?}, ops={:?}", triple.index(), ops);
            Ok(())
        })
    }

    #[traced_test]
    fn test_reconcile_unprocessed_input_only() {
        let mut triple = BatchFileTriple::new_for_test_empty();
        triple.set_index(BatchIndex::from(42u64));
        triple.set_input_path(Some("input_only.json".into()));

        let client_mock = Arc::new(MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap(),
        ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let ect = ExpectedContentType::JsonLines;
        let result = block_on(triple.reconcile_unprocessed(
            client_mock.as_ref(),
            &ect,
            &mock_process_output_fn,
            &mock_process_error_fn,
        ));
        // If there's no error, it means the loop of actions completed or recognized new steps.
        // For input-only, it tries to check for online files. If no error arises, success:
        assert!(result.is_ok(), "Should not fail reconciling an input-only triple");
    }

    #[traced_test]
    fn test_reconcile_unprocessed_input_error_but_mock_processing_fails_action() {
        // We'll simulate a scenario where one action fails:
        // We'll force an error in the "execute_reconciliation_operation" by customizing the triple 
        // or using a mock client that fails. For brevity, let's forcibly set the triple's 
        // move_input_and_error_to_done to fail. We can do so by referencing an invalid workspace.

        struct BadWorkspace;
        impl BatchWorkspaceInterface for BadWorkspace {
            fn workspace_dir(&self) -> &std::path::Path {
                // This is a nonsense path to cause an error in some operation:
                std::path::Path::new("/this/does/not/exist")
            }
        }

        let workspace = Arc::new(BadWorkspace);
        let mut triple = BatchFileTriple::new_for_test_with_workspace(workspace);
        triple.set_input_path(Some("input.json".into()));
        triple.set_error_path(Some("error.json".into()));

        // The recommended steps for input+error => 
        // [EnsureInputRequestIdsMatchErrorRequestIds, ProcessBatchErrorFile, MoveBatchInputAndErrorToTheDoneDirectory].
        // The last step might fail if the directory is invalid. We'll see how the code handles it.

        let client_mock = Arc::new(MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap(),
        ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let ect = ExpectedContentType::JsonLines;

        let result = block_on(triple.reconcile_unprocessed(
            client_mock.as_ref(),
            &ect,
            &mock_process_output_fn,
            &mock_process_error_fn,
        ));
        assert!(result.is_err(), "We expect it to fail due to the invalid workspace path");
        match result.err().unwrap() {
            BatchReconciliationError::ReconciliationFailed { index } => {
                pretty_assert_eq!(index.as_u64(), triple.index().as_u64());
            },
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }
}

