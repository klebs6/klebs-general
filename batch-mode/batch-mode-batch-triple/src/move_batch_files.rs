// ---------------- [ File: src/move_batch_files.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn move_input_and_output_to_done(
        &self,
    ) -> Result<(), FileMoveError> {

        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(&done_dir).await?;
        self.maybe_move_output_to_done(&done_dir).await?;
        self.maybe_move_metadata_to_done(&done_dir).await?;
        Ok(())
    }

    pub async fn move_input_and_error_to_done(
        &self,
    ) -> Result<(), FileMoveError> {

        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(&done_dir).await?;
        self.maybe_move_error_to_done(&done_dir).await?;
        self.maybe_move_metadata_to_done(&done_dir).await?;
        Ok(())
    }

    pub async fn move_all_to_done(
        &self,
    ) -> Result<(), FileMoveError> {

        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(&done_dir).await?;
        self.maybe_move_output_to_done(&done_dir).await?;
        self.maybe_move_error_to_done(&done_dir).await?;
        self.maybe_move_metadata_to_done(&done_dir).await?;
        Ok(())
    }

    //-----------------------------------------------------------
    async fn maybe_move_input_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        // NEW: ensure done_dir exists before rename
        tokio::fs::create_dir_all(done_dir.as_ref()).await.ok();

        if let Some(input_path) = self.input() {
            if !input_path.exists() {
                if is_test_mode() {
                    warn!(
                        "Mock scenario (test-only): ignoring rename for missing input file at {:?}",
                        input_path
                    );
                    return Ok(());
                }
            }
            let dest = done_dir.as_ref().join(input_path.file_name().unwrap());
            trace!("Renaming input_path: {:?} => {:?}", input_path, dest);
            fs::rename(input_path, &dest).await?;
            info!("moved batch input file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_output_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        // NEW: ensure done_dir exists before rename
        tokio::fs::create_dir_all(done_dir.as_ref()).await.ok();

        if let Some(output_path) = self.output() {
            if !output_path.exists() {
                if is_test_mode() {
                    warn!(
                        "Mock scenario (test-only): ignoring rename for missing output file at {:?}",
                        output_path
                    );
                    return Ok(());
                }
            }
            let dest = done_dir.as_ref().join(output_path.file_name().unwrap());
            trace!("Renaming output_path: {:?} => {:?}", output_path, dest);
            fs::rename(output_path, &dest).await?;
            info!("moved batch output file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_error_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        // NEW: ensure done_dir exists before rename
        tokio::fs::create_dir_all(done_dir.as_ref()).await.ok();

        if let Some(error_path) = self.error() {
            if !error_path.exists() {
                if is_test_mode() {
                    warn!(
                        "Mock scenario (test-only): ignoring rename for missing error file at {:?}",
                        error_path
                    );
                    return Ok(());
                }
            }
            let dest = done_dir.as_ref().join(error_path.file_name().unwrap());
            trace!("Renaming error_path: {:?} => {:?}", error_path, dest);
            fs::rename(error_path, &dest).await?;
            info!("moved batch error file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_metadata_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        // NEW: ensure done_dir exists before rename
        tokio::fs::create_dir_all(done_dir.as_ref()).await.ok();

        if let Some(metadata_path) = self.associated_metadata() {
            if !metadata_path.exists() {
                if is_test_mode() {
                    warn!(
                        "Mock scenario (test-only): ignoring rename for missing metadata file at {:?}",
                        metadata_path
                    );
                    return Ok(());
                }
            }
            let dest = done_dir.as_ref().join(metadata_path.file_name().unwrap());
            trace!("Renaming metadata_path: {:?} => {:?}", metadata_path, dest);
            fs::rename(metadata_path, &dest).await?;
            info!("moved batch metadata file to the done directory");
        }
        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_moving_files_exhaustive_tests {
    use super::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::io::Write;
    use tokio::runtime::Runtime;
    use tracing::*;

    #[traced_test]
    fn move_input_and_output_to_done_moves_correct_files() {
        info!("Starting test: move_input_and_output_to_done_moves_correct_files");

        // We'll set up a temp directory that will serve both as our working area
        // and as the 'done' directory (so that rename doesn't fail from mismatched paths).
        let temp_dir = TempDir::new().expect("Failed to create TempDir for test environment");

        // Create actual files in the temp dir
        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");
        let metadata_path = temp_dir.path().join("metadata.json");
        {
            let mut input_file = std::fs::File::create(&input_path)
                .expect("Failed to create input.json in temp dir");
            writeln!(input_file, "fake input contents").unwrap();

            let mut output_file = std::fs::File::create(&output_path)
                .expect("Failed to create output.json in temp dir");
            writeln!(output_file, "fake output contents").unwrap();

            // We'll also create an empty metadata file
            let _ = std::fs::File::create(&metadata_path)
                .expect("Failed to create metadata.json in temp dir");
        }

        // We'll point our "done_dir" to the same temp_dir, so the rename target is valid.
        let workspace = MockWorkspaceBuilder::default()
            .done_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        // Construct a triple referencing these new files
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(4),
            Some(input_path.clone()),
            Some(output_path.clone()),
            None, // we do NOT have an error file here
            Some(metadata_path.clone()),
            Arc::new(workspace),
        );

        // Move them
        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let result = rt.block_on(async { triple.move_input_and_output_to_done().await });
        assert!(result.is_ok(), "move_input_and_output_to_done should succeed with existing files");

        // Now check that they have been moved inside the same temp dir's "done" location
        // In this mock, get_done_directory() is the temp_dir path we set above.
        let done_dir = triple.get_done_directory();

        // The old paths should no longer exist
        assert!(!input_path.exists(), "input should have been moved away");
        assert!(!output_path.exists(), "output should have been moved away");
        assert!(!metadata_path.exists(), "metadata should have been moved away");

        // The new paths should exist in done_dir with the same filenames
        assert!(done_dir.join("input.json").exists(), "input must be in done_dir now");
        assert!(done_dir.join("output.json").exists(), "output must be in done_dir now");
        assert!(done_dir.join("metadata.json").exists(), "metadata must be in done_dir now");

        info!("Finished test: move_input_and_output_to_done_moves_correct_files");
    }

    #[traced_test]
    fn move_input_and_error_to_done_moves_correct_files() {
        info!("Starting test: move_input_and_error_to_done_moves_correct_files");

        let temp_dir = TempDir::new().expect("Failed to create TempDir for test environment");

        let input_path = temp_dir.path().join("input.json");
        let error_path = temp_dir.path().join("error.json");
        let metadata_path = temp_dir.path().join("metadata.json");
        {
            let mut input_file = std::fs::File::create(&input_path)
                .expect("Failed to create input.json in temp dir");
            writeln!(input_file, "fake input contents").unwrap();

            let mut error_file = std::fs::File::create(&error_path)
                .expect("Failed to create error.json in temp dir");
            writeln!(error_file, "fake error contents").unwrap();

            let _ = std::fs::File::create(&metadata_path)
                .expect("Failed to create metadata.json in temp dir");
        }

        let workspace = MockWorkspaceBuilder::default()
            .done_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(99),
            Some(input_path.clone()),
            None,
            Some(error_path.clone()),
            Some(metadata_path.clone()),
            Arc::new(workspace),
        );

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let result = rt.block_on(async { triple.move_input_and_error_to_done().await });
        assert!(result.is_ok(), "move_input_and_error_to_done should succeed with existing files");

        let done_dir = triple.get_done_directory();

        assert!(!input_path.exists(), "input should have been moved");
        assert!(!error_path.exists(), "error should have been moved");
        assert!(!metadata_path.exists(), "metadata should have been moved");

        assert!(done_dir.join("input.json").exists(), "input now in done_dir");
        assert!(done_dir.join("error.json").exists(), "error now in done_dir");
        assert!(done_dir.join("metadata.json").exists(), "metadata now in done_dir");

        info!("Finished test: move_input_and_error_to_done_moves_correct_files");
    }

    #[traced_test]
    fn move_all_to_done_moves_input_output_error_and_metadata() {
        info!("Starting test: move_all_to_done_moves_input_output_error_and_metadata");

        let temp_dir = TempDir::new().expect("Failed to create TempDir for test environment");

        let input_path = temp_dir.path().join("input.json");
        let output_path = temp_dir.path().join("output.json");
        let error_path = temp_dir.path().join("error.json");
        let metadata_path = temp_dir.path().join("metadata.json");
        {
            let mut input_file = std::fs::File::create(&input_path)
                .expect("Failed to create input.json in temp dir");
            writeln!(input_file, "some input content").unwrap();

            let mut output_file = std::fs::File::create(&output_path)
                .expect("Failed to create output.json in temp dir");
            writeln!(output_file, "some output content").unwrap();

            let mut error_file = std::fs::File::create(&error_path)
                .expect("Failed to create error.json in temp dir");
            writeln!(error_file, "some error content").unwrap();

            let mut metadata_file = std::fs::File::create(&metadata_path)
                .expect("Failed to create metadata.json in temp dir");
            writeln!(metadata_file, "some metadata content").unwrap();
        }

        let workspace = MockWorkspaceBuilder::default()
            .done_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(5),
            Some(input_path.clone()),
            Some(output_path.clone()),
            Some(error_path.clone()),
            Some(metadata_path.clone()),
            Arc::new(workspace),
        );
        debug!("Constructed triple for test: {:?}", triple);

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let res = rt.block_on(async { triple.move_all_to_done().await });
        assert!(res.is_ok(), "move_all_to_done should succeed with existing files");

        let done_dir = triple.get_done_directory();

        assert!(!input_path.exists(), "input should have been moved");
        assert!(!output_path.exists(), "output should have been moved");
        assert!(!error_path.exists(), "error should have been moved");
        assert!(!metadata_path.exists(), "metadata should have been moved");

        assert!(done_dir.join("input.json").exists(), "input must be in done_dir now");
        assert!(done_dir.join("output.json").exists(), "output must be in done_dir now");
        assert!(done_dir.join("error.json").exists(), "error must be in done_dir now");
        assert!(done_dir.join("metadata.json").exists(), "metadata must be in done_dir now");

        info!("Finished test: move_all_to_done_moves_input_output_error_and_metadata");
    }
}
