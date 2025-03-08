// ---------------- [ File: src/gather_all_batch_files.rs ]
crate::ix!();

#[async_trait]
pub trait GatherAllBatchTriples: Send + Sync {
    async fn gather_all_batch_triples(
        self: Arc<Self>,
    ) -> Result<Vec<BatchFileTriple>, BatchWorkspaceError>;
}

#[async_trait]
impl<T> GatherAllBatchTriples for T
where
    for<'async_trait> T: LocateBatchFiles + FindExistingBatchFileIndices + Send + Sync + 'async_trait,
{
    async fn gather_all_batch_triples(
        self: Arc<Self>,
    ) -> Result<Vec<BatchFileTriple>, BatchWorkspaceError>
    {
        trace!("gathering all batch triples across known indices");

        // First, obtain the set of all existing batch indices in the given directory.
        let indices = self.clone().find_existing_batch_file_indices().await?;
        debug!("found batch indices: {:?}", indices);

        let mut batch_files = Vec::new();

        for index in indices {
            if let Some(batch) = self.clone().locate_batch_files(&index).await? {
                trace!("found a triple for index {:?}", index);
                batch_files.push(batch);
            }
        }

        batch_files.sort();
        info!("final list of batch file triples: {:?}", batch_files);

        Ok(batch_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[traced_test]
    async fn test_gather_all_batch_files_all_present() -> Result<(), BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir = workspace.workdir();

        println!("BatchWorkspace directory: {:?}", workdir);

        // Setup batch files with a few indices
        let indices = vec![1, 2, 3];
        for index in &indices {
            let input_path = workdir.join(format!("batch_input_{}.jsonl", index));
            let output_path = workdir.join(format!("batch_output_{}.jsonl", index));
            let error_path = workdir.join(format!("batch_error_{}.jsonl", index));

            fs::write(&input_path, "input data").await?;
            fs::write(&output_path, "output data").await?;
            fs::write(&error_path, "error data").await?;

            // Verify files exist
            fs::metadata(&input_path).await?;
            fs::metadata(&output_path).await?;
            fs::metadata(&error_path).await?;
        }

        // Call the function under test
        let batch_files = workspace.gather_all_batch_triples().await?;

        // Assert that we get the correct number of batch files
        assert_eq!(batch_files.len(), indices.len());

        // Additional assertions to check the contents of each BatchFileTriple
        for (i, batch) in batch_files.iter().enumerate() {
            assert_eq!(*batch.index(), BatchIndex::Usize(indices[i]));
            assert!(batch.input().is_some());
            assert!(batch.output().is_some());
            assert!(batch.error().is_some());
        }

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_partial_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir = workspace.workdir();

        // Create different combinations of partial batch files
        let input_only_path = workdir.join("batch_input_1.jsonl");
        fs::write(&input_only_path, "input data").await?;

        let input_output_path_1 = workdir.join("batch_input_2.jsonl");
        let input_output_path_2 = workdir.join("batch_output_2.jsonl");
        fs::write(&input_output_path_1, "input data").await?;
        fs::write(&input_output_path_2, "output data").await?;

        // Call the function under test
        let batch_files = workspace.gather_all_batch_triples().await?;

        // Assert correct batch files were collected
        assert_eq!(batch_files.len(), 2);

        for batch in batch_files {
            match batch.index() {
                BatchIndex::Usize(1) => {
                    assert!(batch.input().is_some());
                    assert!(batch.output().is_none());
                    assert!(batch.error().is_none());
                },
                BatchIndex::Usize(2) => {
                    assert!(batch.input().is_some());
                    assert!(batch.output().is_some());
                    assert!(batch.error().is_none());
                },
                _ => panic!("Unexpected batch index"),
            }
        }

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_none_present() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;

        // Directory is empty
        let batch_files = workspace.gather_all_batch_triples().await?;

        // Assert that no batch files were found
        assert!(batch_files.is_empty());

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_non_existent_directory() 
        -> Result<(), BatchWorkspaceError> 
    {
        use tempfile::tempdir;
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        use tokio::fs;

        // Create a temporary directory
        let temp_dir = tempdir().map_err(BatchWorkspaceError::IoError)?;

        // Set the permissions to read-only
        let permissions = Permissions::from_mode(0o555); // Read and execute permissions, no write
        fs::set_permissions(temp_dir.path(), permissions).await.map_err(BatchWorkspaceError::IoError)?;

        // Attempt to create a workspace within this directory
        let path = temp_dir.path().join("subdir");
        let result = BatchWorkspace::new_in(&path).await;

        // Assert that an error is returned
        assert!(result.is_err());

        // Optionally, check that the error is due to permission denied
        if let Err(BatchWorkspaceError::IoError(ref e)) = result {
            assert_eq!(e.kind(), std::io::ErrorKind::PermissionDenied);
        } else {
            panic!("Expected an IoError due to permission denied");
        }

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_malformed_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Create malformed files that should be ignored
        let malformed_file_1 = workdir.join("malformed_file.jsonl");
        let malformed_file_2 = workdir.join("batch_x_input.jsonl");
        fs::write(&malformed_file_1, "some data").await?;
        fs::write(&malformed_file_2, "some data").await?;

        // Create valid batch files
        let valid_input_path = workdir.join("batch_input_3.jsonl");
        fs::write(&valid_input_path, "input data").await?;

        // Call the function under test
        let batch_files = workspace.gather_all_batch_triples().await?;

        // Assert that only valid batch files were detected
        assert_eq!(batch_files.len(), 1);
        assert_eq!(*batch_files[0].index(), BatchIndex::Usize(3));
        assert!(batch_files[0].input().is_some());

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_concurrency() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Create valid batch files
        for index in 1..=10 {
            let input_path = workdir.join(format!("batch_input_{}.jsonl", index));
            fs::write(&input_path, "input data").await?;
        }

        // Launch multiple async calls concurrently
        let futures = vec![
            workspace.gather_all_batch_triples(),
            workspace.gather_all_batch_triples(),
            workspace.gather_all_batch_triples(),
        ];

        let results = futures::future::join_all(futures).await;

        // Assert all calls returned correct results
        for result in results {
            assert!(result.is_ok());
            let batch_files = result.unwrap();
            assert_eq!(batch_files.len(), 10);
        }

        Ok(())
    }

    #[traced_test]
    async fn test_gather_all_batch_files_duplicate_indices() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Create files with duplicated indices
        let input_path_1 = workdir.join("batch_input_4.jsonl");
        let input_path_2 = workdir.join("batch_input_4_duplicate.jsonl");
        fs::write(&input_path_1, "input data 1").await?;
        fs::write(&input_path_2, "input data 2").await?;

        // Call the function under test
        let batch_files = workspace.gather_all_batch_triples().await?;

        // Assert that only the first valid batch index is present
        assert_eq!(batch_files.len(), 1);
        assert_eq!(*batch_files[0].index(), BatchIndex::Usize(4));
        assert!(batch_files[0].input().is_some());

        Ok(())
    }
}
