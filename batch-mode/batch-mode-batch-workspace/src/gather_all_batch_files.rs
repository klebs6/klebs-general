// ---------------- [ File: src/gather_all_batch_files.rs ]
crate::ix!();

#[async_trait]
impl<T> GatherAllBatchTriples for T
where
    for<'async_trait> T: LocateBatchFiles + FindExistingBatchFileIndices + Send + Sync + 'async_trait,
    BatchWorkspaceError: From<<T as LocateBatchFiles>::Error>,
    BatchWorkspaceError: From<<T as FindExistingBatchFileIndices>::Error>,
{
    type Error = BatchWorkspaceError;
    async fn gather_all_batch_triples(
        self: Arc<Self>,
    ) -> Result<Vec<BatchFileTriple>, Self::Error>
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
mod gather_all_batch_triples_exhaustive_tests {
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
            workspace.clone().gather_all_batch_triples(),
            workspace.clone().gather_all_batch_triples(),
            workspace.clone().gather_all_batch_triples(),
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

    #[traced_test]
    async fn returns_empty_when_no_files_found() {
        info!("Starting test: returns_empty_when_no_files_found");
        // We'll create a new temporary workspace, ensuring no batch files exist yet
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");

        // gather_all_batch_triples should yield an empty list
        let triples = workspace.clone().gather_all_batch_triples().await.expect("Should succeed");
        debug!("Resulting triples: {:?}", triples);
        assert!(triples.is_empty(), "Expected an empty list of batch file triples");

        info!("Finished test: returns_empty_when_no_files_found");
    }

    #[traced_test]
    async fn returns_all_valid_indices_with_single_file_each() {
        info!("Starting test: returns_all_valid_indices_with_single_file_each");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let workdir   = workspace.workdir();

        // We'll create a few indices: 1,2,3 each with exactly one input file
        let indices = [1, 2, 3];
        for idx in &indices {
            let filename = format!("batch_input_{}.jsonl", idx);
            fs::write(workdir.join(&filename), format!("input file for index {}", idx))
                .await
                .expect("Failed to write input file");
        }

        // gather_all_batch_triples should find all these indices (1,2,3)
        let triples = workspace
            .clone()
            .gather_all_batch_triples()
            .await
            .expect("Should succeed in reading indices and locating batch files");
        debug!("Gathered triples: {:?}", triples);

        assert_eq!(triples.len(), indices.len());
        for triple in &triples {
            if let BatchIndex::Usize(u) = triple.index() {
                assert!(
                    indices.contains(u),
                    "Found unexpected index: {} in gathered list",
                    u
                );
            } else {
                panic!("Expected only Usize indices, got something else");
            }
        }

        info!("Finished test: returns_all_valid_indices_with_single_file_each");
    }

    #[traced_test]
    async fn includes_partial_sets_of_files() {
        info!("Starting test: includes_partial_sets_of_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll create:
        //  - batch_input_10.jsonl (no output/error)
        //  - batch_input_11.jsonl + batch_output_11.jsonl
        //  - batch_input_12.jsonl + batch_output_12.jsonl + batch_error_12.jsonl
        // That should result in 3 distinct gathered triples.
        let combos = vec![
            (10, vec!["input"]),
            (11, vec!["input", "output"]),
            (12, vec!["input", "output", "error"]),
        ];

        for (idx, types) in combos {
            for t in types {
                let filename = format!("batch_{}_{}.jsonl", t, idx);
                fs::write(wd.join(filename), b"test content").await.unwrap();
            }
        }

        let all = workspace
            .clone()
            .gather_all_batch_triples()
            .await
            .expect("Should succeed scanning partial sets of files");

        debug!("Result => Found {} triples: {:?}", all.len(), all);
        assert_eq!(
            all.len(),
            3,
            "Should find exactly 3 distinct batch triples for indices 10,11,12"
        );

        // Check that each index is indeed found
        let found_indices: Vec<_> = all.iter().map(|b| b.index().clone()).collect();
        let mut found_usizes = Vec::new();
        for idx in found_indices {
            if let BatchIndex::Usize(u) = idx {
                found_usizes.push(u);
            } else {
                panic!("Expected only Usize indices for this test");
            }
        }
        found_usizes.sort();
        assert_eq!(found_usizes, vec![10, 11, 12]);

        info!("Finished test: includes_partial_sets_of_files");
    }

    #[traced_test]
    async fn ignores_invalid_filenames_while_still_including_valid_ones() {
        info!("Starting test: ignores_invalid_filenames_while_still_including_valid_ones");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // Create some valid batch files for index=42
        fs::write(wd.join("batch_input_42.jsonl"), b"input data for 42").await.unwrap();
        fs::write(wd.join("batch_error_42.jsonl"), b"error data for 42").await.unwrap();

        // Create some invalid named files that partially match but not the correct capturing group
        fs::write(wd.join("batch_foo_42.jsonl"), b"nonsense").await.unwrap();
        fs::write(wd.join("batch_42.jsonl"), b"missing type").await.unwrap();
        fs::write(wd.join("foo_batch_input_42.jsonl"), b"wrong prefix").await.unwrap();

        // Also a random file that is nowhere near the pattern
        fs::write(wd.join("random_notes.txt"), b"some random text").await.unwrap();

        let all = workspace
            .clone()
            .gather_all_batch_triples()
            .await
            .expect("Should succeed ignoring invalid files");

        debug!("gather_all_batch_triples => {:?}", all);
        assert_eq!(
            all.len(),
            1,
            "We have only 1 valid index (42) with recognized file types"
        );

        let triple = &all[0];
        assert_eq!(*triple.index(), BatchIndex::Usize(42));
        assert!(triple.input().is_some());
        assert!(triple.error().is_some());
        assert!(triple.output().is_none());
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: ignores_invalid_filenames_while_still_including_valid_ones");
    }

    #[traced_test]
    async fn indexes_are_sorted_in_final_output() {
        info!("Starting test: indexes_are_sorted_in_final_output");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll add indices 3,1,2 out of order
        for i in [3,1,2] {
            fs::write(
                wd.join(format!("batch_input_{}.jsonl", i)),
                format!("batch input for index {}", i)
            ).await.unwrap();
        }

        // gather
        let all = workspace
            .clone()
            .gather_all_batch_triples()
            .await
            .expect("Should succeed scanning out-of-order indices");

        debug!("Resulting list => {:?}", all);
        // Expect them sorted: index 1,2,3 in ascending order
        assert_eq!(all.len(), 3, "We created exactly 3 indices");
        let mut last = 0;
        for triple in &all {
            if let BatchIndex::Usize(u) = triple.index() {
                assert!(*u > last, "Indices not sorted properly");
                last = *u;
            } else {
                panic!("Expected only Usize indices for this test");
            }
        }
        info!("Finished test: indexes_are_sorted_in_final_output");
    }

    #[traced_test]
    async fn concurrency_test_across_multiple_indices() {
        info!("Starting test: concurrency_test_across_multiple_indices");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll produce multiple indices each with an input file
        let indices = [5,6,7,8,9];
        for i in &indices {
            let name = format!("batch_input_{}.jsonl", i);
            fs::write(wd.join(name), b"concurrency test data").await.unwrap();
        }

        // We'll run gather_all_batch_triples concurrently
        let arc_ws = workspace.clone();
        let mut tasks = Vec::new();
        for i in 0..5 {
            let ws_clone = arc_ws.clone();
            tasks.push(tokio::spawn(async move {
                debug!("Task #{} gathering all batch triples now", i);
                ws_clone.gather_all_batch_triples().await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(Ok(triples)) => {
                    debug!("Task {} => gathered {} triples", i, triples.len());
                    assert_eq!(triples.len(), indices.len(), "We expect exactly 5 indices");
                }
                Ok(Err(e)) => panic!("Task {} => unexpected error: {:?}", i, e),
                Err(e)     => panic!("Task {} => join error: {:?}", i, e),
            }
        }
        info!("Finished test: concurrency_test_across_multiple_indices");
    }

    #[traced_test]
    async fn gracefully_handles_errors_from_find_existing_batch_file_indices() {
        info!("Starting test: gracefully_handles_errors_from_find_existing_batch_file_indices");
        // We'll create a custom workspace that fails in find_existing_batch_file_indices for demonstration.
        // Instead, let's forcibly create a directory with no read permissions.

        let tmp = tempdir().expect("Failed to create base tempdir");
        let dir_path = tmp.path().join("inaccessible");
        std::fs::create_dir_all(&dir_path).expect("Failed to create test subdir");

        // Make the directory read-only, on Unix:
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dir_path).unwrap().permissions();
            perms.set_mode(0o000);
            std::fs::set_permissions(&dir_path, perms).unwrap();
        }

        // Now let's create a workspace in that dir
        let workspace_res = BatchWorkspace::new_in(&dir_path).await;
        // We expect that we might not even get far. If we do succeed in creating
        // the workspace, gather_all_batch_triples will likely fail to read it:
        match workspace_res {
            Ok(ws) => {
                // Attempt to gather
                let r = ws.clone().gather_all_batch_triples().await;
                debug!("Result from gather_all_batch_triples in read-only directory: {:?}", r);
                assert!(r.is_err(), "We expect an error from reading an inaccessible directory");
            }
            Err(e) => {
                // This is also acceptable, it means new_in already failed.
                warn!("new_in() failed as expected: {:?}", e);
            }
        }

        info!("Finished test: gracefully_handles_errors_from_find_existing_batch_file_indices");
    }

    #[traced_test]
    async fn handles_mixed_usize_and_uuid_indices() {
        info!("Starting test: handles_mixed_usize_and_uuid_indices");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll have an integer index=100 plus a UUID index.
        // We'll store the raw Uuid here to compare in the match.
        let raw_uuid = uuid::Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d479")
            .expect("Invalid UUID in test data");
        let idx_usize = 100;
        let idx_uuid  = BatchIndex::Uuid(raw_uuid);

        // Write some files
        fs::write(wd.join(format!("batch_input_{}.jsonl", idx_usize)), b"usize input").await.unwrap();
        fs::write(wd.join(format!("batch_output_{}.jsonl", raw_uuid)), b"uuid output").await.unwrap();

        // gather
        let all = workspace
            .clone()
            .gather_all_batch_triples()
            .await
            .expect("Should succeed gathering mixed-type indices");

        debug!("found {} batch file triple(s): {:?}", all.len(), all);
        assert_eq!(all.len(), 2, "We have 2 distinct indices, one usize, one uuid");

        // Check them
        let mut found_usize = false;
        let mut found_uuid  = false;
        for triple in &all {
            match triple.index() {
                BatchIndex::Usize(u) if *u == idx_usize => {
                    found_usize = true;
                    assert!(triple.input().is_some());
                    assert!(triple.output().is_none());
                    assert!(triple.error().is_none());
                }
                BatchIndex::Uuid(u) if *u == raw_uuid => {
                    found_uuid = true;
                    assert!(triple.output().is_some());
                    assert!(triple.input().is_none());
                    assert!(triple.error().is_none());
                }
                other => panic!("Unexpected index in the gathered results: {:?}", other),
            }
        }

        assert!(found_usize, "Did not find the expected usize index triple");
        assert!(found_uuid,  "Did not find the expected UUID index triple");
        info!("Finished test: handles_mixed_usize_and_uuid_indices");
    }
}
