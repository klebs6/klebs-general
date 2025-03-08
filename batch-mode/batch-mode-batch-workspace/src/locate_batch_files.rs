// ---------------- [ File: src/locate_batch_files.rs ]
crate::ix!();

#[async_trait]
impl<T> LocateBatchFiles for T
where
    for<'async_trait> T: BatchWorkspaceInterface + Send + Sync + 'async_trait,
{
    type Error = BatchWorkspaceError;
    async fn locate_batch_files(
        self:  Arc<Self>,
        index: &BatchIndex
    ) -> Result<Option<BatchFileTriple>, Self::Error> {
        trace!("attempting to locate batch files for index: {:?}", index);

        // Get the regex pattern for the specified index to match filenames
        let file_pattern = index.file_pattern();

        let mut input               = None;
        let mut output              = None;
        let mut error               = None;
        let mut associated_metadata = None;

        let mut entries = fs::read_dir(self.workdir()).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Get filename as a &str
            let filename = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => {
                    trace!("skipping a file with non-UTF8 name: {:?}", path);
                    continue;
                }
            };

            // Use the precompiled regex pattern to match filenames
            let captures = match file_pattern.captures(filename) {
                Some(captures) => captures,
                None => {
                    debug!("filename does not match the expected pattern: {:?}", filename);
                    continue;
                }
            };

            // Extract the type of the file from the capture group
            let file_type = captures.get(1).map(|m| m.as_str());

            match file_type {
                Some("input") => {
                    if input.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple input files found"
                        ).into());
                    }
                    debug!("found input file: {:?}", path);
                    input = Some(path);
                }
                Some("output") => {
                    if output.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple output files found"
                        ).into());
                    }
                    debug!("found output file: {:?}", path);
                    output = Some(path);
                }
                Some("error") => {
                    if error.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple error files found"
                        ).into());
                    }
                    debug!("found error file: {:?}", path);
                    error = Some(path);
                }
                Some("metadata") => {
                    if associated_metadata.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple associated_metadata files found"
                        ).into());
                    }
                    debug!("found associated_metadata file: {:?}", path);
                    associated_metadata = Some(path);
                }
                _ => {
                    trace!("skipping unrecognized file type: {:?}", filename);
                    continue;
                }
            }
        }

        if input.is_none() && output.is_none() && error.is_none() && associated_metadata.is_none() {
            debug!("no batch files found in directory for index {:?}: {:?}", index, self.workdir());
            Ok(None)
        } else {
            debug!(
                "batch files located for index {:?} - input: {:?}, output: {:?}, error: {:?}, metadata: {:?}",
                index, input, output, error, associated_metadata
            );
            Ok(Some(BatchFileTriple::new_direct(
                        index, 
                        input, 
                        output, 
                        error, 
                        associated_metadata, 
                        self.clone()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[traced_test]
    async fn test_locate_batch_files_usize() -> Result<(),BatchWorkspaceError> {

        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        fs::write(workdir.join("batch_input_4.jsonl"), b"test").await?;
        fs::write(workdir.join("batch_output_4.jsonl"), b"test").await?;
        fs::write(workdir.join("batch_error_4.jsonl"), b"test").await?;

        let batch_files = workspace.clone().locate_batch_files(&BatchIndex::Usize(4)).await?.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        assert_eq!(*batch_files.output(), Some(workdir.join("batch_output_4.jsonl")));
        assert_eq!(*batch_files.error(), Some(workdir.join("batch_error_4.jsonl")));

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_uuid() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        fs::write(workdir.join(format!("batch_input_{}.jsonl", uuid)), b"test").await?;
        fs::write(workdir.join(format!("batch_output_{}.jsonl", uuid)), b"test").await?;

        let batch_files = workspace.clone().locate_batch_files(&BatchIndex::from_uuid_str(uuid)?).await?.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join(format!("batch_input_{}.jsonl", uuid))));
        assert_eq!(*batch_files.output(), Some(workdir.join(format!("batch_output_{}.jsonl", uuid))));
        assert_eq!(*batch_files.error(), None);

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_no_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;

        let batch_files = workspace.locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(batch_files.is_none());

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_ignores_invalid_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Write one valid input file
        fs::write(workdir.join("batch_input_4.jsonl"), b"test").await?;
        // Write one file that doesn't match the pattern
        fs::write(workdir.join("batch_input_4_duplicate.jsonl"), b"test").await?;

        let result = workspace.clone().locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(result.is_some(), "Expected to find the valid batch input file");

        let batch_files = result.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        assert!(batch_files.output().is_none());
        assert!(batch_files.error().is_none());

        Ok(())
    }
}
