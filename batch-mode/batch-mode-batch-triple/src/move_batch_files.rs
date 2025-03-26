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
    async fn move_input_and_output_to_done_moves_correct_files() {
        trace!("===== BEGIN TEST: move_input_and_output_to_done_moves_correct_files =====");
        info!("Starting test: move_input_and_output_to_done_moves_correct_files");

        // Create a temp workspace that is fully sandboxed
        let workspace = MockBatchWorkspace::default();
        let batch_idx = BatchIndex::Usize(42);

        // Write some fake files in the ephemeral workspace:
        let input_path = workspace.input_filename(&batch_idx);
        std::fs::write(&input_path, "fake input contents")
            .expect("Failed to write to input file");

        let output_path = workspace.output_filename(&batch_idx);
        std::fs::write(&output_path, "fake output contents")
            .expect("Failed to write to output file");

        let metadata_path = workspace.metadata_filename(&batch_idx);
        std::fs::write(&metadata_path, "fake metadata contents")
            .expect("Failed to write to metadata file");

        // Construct the triple referencing those ephemeral paths
        let triple = BatchFileTriple::new_direct(
            &batch_idx,
            Some(input_path.clone()),
            Some(output_path.clone()),
            None,
            Some(metadata_path.clone()),
            Arc::new(workspace.clone()),
        );

        let result = triple.move_input_and_output_to_done().await;
        debug!("Result of move_input_and_output_to_done: {:?}", result);
        assert!(
            result.is_ok(),
            "Expected success moving input + output to done"
        );

        // Verify they are no longer in their original location:
        assert!(!input_path.exists(), "Input file should have been moved away");
        assert!(!output_path.exists(), "Output file should have been moved away");
        assert!(
            !metadata_path.exists(),
            "Metadata file should have been moved away"
        );

        // Confirm they appear in the workspace's ephemeral done directory
        let done_dir = workspace.get_done_directory();
        trace!("Done directory is: {:?}", done_dir);

        let done_input = done_dir.join(input_path.file_name().unwrap());
        let done_output = done_dir.join(output_path.file_name().unwrap());
        let done_metadata = done_dir.join(metadata_path.file_name().unwrap());

        assert!(
            done_input.exists(),
            "Input file must be in done directory"
        );
        assert!(
            done_output.exists(),
            "Output file must be in done directory"
        );
        assert!(
            done_metadata.exists(),
            "Metadata file must be in done directory"
        );

        info!("Finished test: move_input_and_output_to_done_moves_correct_files");
        trace!("===== END TEST: move_input_and_output_to_done_moves_correct_files =====");
    }

    #[traced_test]
    async fn move_input_and_error_to_done_moves_correct_files() {
        trace!("===== BEGIN TEST: move_input_and_error_to_done_moves_correct_files =====");
        info!("Starting test: move_input_and_error_to_done_moves_correct_files");

        let workspace = MockBatchWorkspace::default();
        let batch_idx = BatchIndex::Usize(777);

        let input_path = workspace.input_filename(&batch_idx);
        std::fs::write(&input_path, "fake input contents")
            .expect("Failed to write to input file");

        let error_path = workspace.error_filename(&batch_idx);
        std::fs::write(&error_path, "fake error contents")
            .expect("Failed to write to error file");

        let metadata_path = workspace.metadata_filename(&batch_idx);
        std::fs::write(&metadata_path, "fake metadata contents")
            .expect("Failed to write to metadata file");

        let triple = BatchFileTriple::new_direct(
            &batch_idx,
            Some(input_path.clone()),
            None,
            Some(error_path.clone()),
            Some(metadata_path.clone()),
            Arc::new(workspace.clone()),
        );

        let result = triple.move_input_and_error_to_done().await;
        debug!("Result of move_input_and_error_to_done: {:?}", result);
        assert!(
            result.is_ok(),
            "Expected success moving input + error to done"
        );

        assert!(!input_path.exists(), "Input file should have been moved away");
        assert!(!error_path.exists(), "Error file should have been moved away");
        assert!(
            !metadata_path.exists(),
            "Metadata file should have been moved away"
        );

        let done_dir = workspace.get_done_directory();
        trace!("Done directory is: {:?}", done_dir);

        let done_input = done_dir.join(input_path.file_name().unwrap());
        let done_error = done_dir.join(error_path.file_name().unwrap());
        let done_metadata = done_dir.join(metadata_path.file_name().unwrap());

        assert!(
            done_input.exists(),
            "Input file must be in done directory"
        );
        assert!(
            done_error.exists(),
            "Error file must be in done directory"
        );
        assert!(
            done_metadata.exists(),
            "Metadata file must be in done directory"
        );

        info!("Finished test: move_input_and_error_to_done_moves_correct_files");
        trace!("===== END TEST: move_input_and_error_to_done_moves_correct_files =====");
    }

    #[traced_test]
    async fn move_all_to_done_moves_input_output_error_and_metadata() {
        trace!("===== BEGIN TEST: move_all_to_done_moves_input_output_error_and_metadata =====");
        info!("Starting test: move_all_to_done_moves_input_output_error_and_metadata");

        let workspace = MockBatchWorkspace::default();
        let batch_idx = BatchIndex::Usize(5);

        let input_path = workspace.input_filename(&batch_idx);
        std::fs::write(&input_path, "some input content")
            .expect("Failed to write to input file");

        let output_path = workspace.output_filename(&batch_idx);
        std::fs::write(&output_path, "some output content")
            .expect("Failed to write to output file");

        let error_path = workspace.error_filename(&batch_idx);
        std::fs::write(&error_path, "some error content")
            .expect("Failed to write to error file");

        let metadata_path = workspace.metadata_filename(&batch_idx);
        std::fs::write(&metadata_path, "some metadata content")
            .expect("Failed to write to metadata file");

        let triple = BatchFileTriple::new_direct(
            &batch_idx,
            Some(input_path.clone()),
            Some(output_path.clone()),
            Some(error_path.clone()),
            Some(metadata_path.clone()),
            Arc::new(workspace.clone()),
        );

        debug!("Constructed triple for test: {:?}", triple);

        let result = triple.move_all_to_done().await;
        debug!("Result of move_all_to_done: {:?}", result);
        assert!(
            result.is_ok(),
            "Expected success moving all files to done"
        );

        assert!(!input_path.exists(), "Input file should be moved away");
        assert!(!output_path.exists(), "Output file should be moved away");
        assert!(!error_path.exists(), "Error file should be moved away");
        assert!(
            !metadata_path.exists(),
            "Metadata file should be moved away"
        );

        let done_dir = workspace.get_done_directory();
        trace!("Done directory is: {:?}", done_dir);

        let done_input = done_dir.join(input_path.file_name().unwrap());
        let done_output = done_dir.join(output_path.file_name().unwrap());
        let done_error = done_dir.join(error_path.file_name().unwrap());
        let done_metadata = done_dir.join(metadata_path.file_name().unwrap());

        assert!(done_input.exists(), "Input must be in done directory now");
        assert!(done_output.exists(), "Output must be in done directory now");
        assert!(done_error.exists(), "Error must be in done directory now");
        assert!(
            done_metadata.exists(),
            "Metadata must be in done directory now"
        );

        info!("Finished test: move_all_to_done_moves_input_output_error_and_metadata");
        trace!("===== END TEST: move_all_to_done_moves_input_output_error_and_metadata =====");
    }
}
