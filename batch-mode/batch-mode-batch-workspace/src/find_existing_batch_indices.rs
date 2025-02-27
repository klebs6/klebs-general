// ---------------- [ File: src/find_existing_batch_indices.rs ]
crate::ix!();

impl BatchWorkspace {

    /// Scans the directory and extracts indices from filenames matching the pattern.
    pub async fn find_existing_batch_file_indices(self: &Arc<Self>) 
        -> Result<HashSet<BatchIndex>,BatchWorkspaceError> 
    {
        let workdir = self.workdir();

        // Regex to match filenames of the form batch_{input,output,error}_N.jsonl
        let file_pattern = Regex::new(r"batch_(input|output|error)_(\d+|[a-f0-9\-]{36})\.jsonl$")
            .expect("Invalid regex pattern");

        let mut indices = HashSet::new();

        let mut dir_entries = fs::read_dir(workdir).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|name| name.to_str()) {
                if let Some(captures) = file_pattern.captures(filename) {
                    if let Some(index_match) = captures.get(2) {
                        let index_str = index_match.as_str();
                        let index = if let Ok(num) = index_str.parse::<usize>() {
                            BatchIndex::Usize(num)
                        } else {
                            BatchIndex::from_uuid_str(index_str)?
                        };
                        indices.insert(index);
                    }
                }
            }
        }

        Ok(indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_find_indices() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_mock().await?;

        let indices = workspace.find_existing_batch_file_indices().await?;
        let mut expected_indices = HashSet::new();
        expected_indices.insert(BatchIndex::Usize(0));
        expected_indices.insert(BatchIndex::Usize(1));
        expected_indices.insert(BatchIndex::Usize(12345));
        expected_indices.insert(BatchIndex::from_uuid_str("550e8400-e29b-41d4-a716-446655440000").unwrap());
        expected_indices.insert(BatchIndex::from_uuid_str("f47ac10b-58cc-4372-a567-0e02b2c3d479").unwrap());

        assert_eq!(indices, expected_indices);

        workspace.cleanup_if_temporary().await
    }
}
