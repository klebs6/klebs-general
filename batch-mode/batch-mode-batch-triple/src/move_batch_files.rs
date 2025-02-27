// ---------------- [ File: src/move_batch_files.rs ]
crate::ix!();

impl BatchFileTriple {

    async fn maybe_move_input_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        if let Some(input_path) = self.input() {
            let dest = done_dir.as_ref().join(input_path.file_name().unwrap());
            fs::rename(input_path, dest).await?;
            info!("moved batch input file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_output_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        if let Some(output_path) = self.output() {
            let dest = done_dir.as_ref().join(output_path.file_name().unwrap());
            fs::rename(output_path, dest).await?;
            info!("moved batch output file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_error_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        if let Some(error_path) = self.error() {
            let dest = done_dir.as_ref().join(error_path.file_name().unwrap());
            fs::rename(error_path, dest).await?;
            info!("moved batch error file to the done directory");
        }
        Ok(())
    }

    async fn maybe_move_metadata_to_done(
        &self,
        done_dir: impl AsRef<Path>,
    ) -> Result<(), FileMoveError> {

        if let Some(metadata_path) = self.associated_metadata() {
            let dest = done_dir.as_ref().join(metadata_path.file_name().unwrap());
            fs::rename(metadata_path, dest).await?;
            info!("moved batch metadata file to the done directory");
        }
        Ok(())
    }

    pub async fn move_input_and_output_to_done(
        &self,
    ) -> Result<(), FileMoveError> {

        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(done_dir).await?;
        self.maybe_move_output_to_done(done_dir).await?;
        self.maybe_move_metadata_to_done(done_dir).await?;
        Ok(())
    }

    pub async fn move_input_and_error_to_done(
        &self,
    ) -> Result<(), FileMoveError> {

        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(done_dir).await?;
        self.maybe_move_error_to_done(done_dir).await?;
        self.maybe_move_metadata_to_done(done_dir).await?;
        Ok(())
    }

    pub async fn move_all_to_done(
        &self,
    ) -> Result<(), FileMoveError> {
        let done_dir = self.get_done_directory();
        self.maybe_move_input_to_done(done_dir).await?;
        self.maybe_move_output_to_done(done_dir).await?;
        self.maybe_move_error_to_done(done_dir).await?;
        self.maybe_move_metadata_to_done(done_dir).await?;
        Ok(())
    }
}
