// ---------------- [ File: src/workspace_interface.rs ]
crate::ix!();

pub trait BatchWorkspaceInterface
: GetInputFilenameAtIndex
+ GetTargetDir
+ GetTargetDirectoryFiles
+ FindSimilarTargetPath
+ GetOutputFilenameAtIndex
+ GetErrorFilenameAtIndex
+ GetMetadataFilenameAtIndex
+ GetDoneDirectory
+ GetFailedJsonRepairsDir
+ GetFailedItemsDir
+ GetTextStoragePath
+ GetWorkdir
+ Send
+ Sync
+ Debug
+ GetTargetPath<Item = Arc<dyn GetTargetPathForAIExpansion + Send + Sync + 'static>>
{}

pub trait GetTargetDir {
    fn get_target_dir(&self) -> PathBuf;
}

//--------------------------------------------------
pub trait GetTargetDirectoryFiles {
    fn get_target_directory_files(&self) -> Vec<PathBuf>;
}

impl<W> GetTargetDirectoryFiles for W 
where W: GetTargetDir
{
    fn get_target_directory_files(&self) -> Vec<PathBuf> {
        // Example implementation: scan the target directory for existing files
        std::fs::read_dir(&self.get_target_dir())
            .unwrap()
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect()
    }
}

//--------------------------------------------------
pub trait FindSimilarTargetPath {
    fn find_similar_target_path(&self, target_path: &Path) -> Option<PathBuf>;
}

impl<W> FindSimilarTargetPath for W 
where W: GetTargetDirectoryFiles
{
    fn find_similar_target_path(&self, target_path: &Path) -> Option<PathBuf> {

        use strsim::levenshtein;

        let existing_paths = self.get_target_directory_files();
        let target_str     = target_path.to_string_lossy();

        existing_paths
            .iter()
            .find(|&existing| levenshtein(&target_str, &existing.to_string_lossy()) <= 2)
            .cloned()
    }
}

//--------------------------------------------------
pub trait GetInputFilenameAtIndex {
    fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf;

}
pub trait GetOutputFilenameAtIndex {
    fn output_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
}

pub trait GetErrorFilenameAtIndex {
    fn error_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
}

pub trait GetMetadataFilenameAtIndex {
    fn metadata_filename(&self, batch_idx: &BatchIndex) -> PathBuf;
}

pub trait GetDoneDirectory {
    fn get_done_directory(&self) -> &PathBuf;
}

pub trait GetTargetPath {
    type Item;
    fn target_path(
        &self,
        item:            &Self::Item, 
        expected_content_type: &ExpectedContentType
    ) -> PathBuf;
}

pub trait GetFailedJsonRepairsDir {
    fn failed_json_repairs_dir(&self) -> PathBuf;
}

pub trait GetFailedItemsDir {
    fn failed_items_dir(&self) -> PathBuf;
}

pub trait GetTextStoragePath {
    fn text_storage_path(&self, batch_idx: &BatchIndex) -> PathBuf;
}

pub trait GetWorkdir {
    fn workdir(&self) -> PathBuf;
}

pub trait GetTargetPathForAIExpansion {

    fn target_path_for_ai_json_expansion(
        &self, 
        target_dir:            &Path,
        expected_content_type: &ExpectedContentType,

    ) -> PathBuf;
}

impl<T:Named> GetTargetPathForAIExpansion for T {

    fn target_path_for_ai_json_expansion(
        &self, 
        target_dir:            &Path,
        _expected_content_type: &ExpectedContentType,

    ) -> PathBuf {

        // Convert 'token_name' to snake_case
        let snake_token_name = to_snake_case(&self.name());

        // Determine the output filename based on custom_id
        // You can customize this as needed, e.g., using token names
        let filename = format!("{}.json", snake_token_name);

        target_dir.to_path_buf().join(filename)
    }
}

//-------------------------------------------------------
pub trait HasAssociatedOutputName {
    fn associated_output_name(&self) -> std::borrow::Cow<'_, str>;
}

pub trait GetTargetPathForAIExpansionFromSeed {

    fn target_path_for_ai_json_expansion_from_seed(
        &self, 
        target_dir:            &Path,
        expected_content_type: &ExpectedContentType,

    ) -> PathBuf;
}

impl<T:Named+HasAssociatedOutputName> GetTargetPathForAIExpansionFromSeed for T {

    fn target_path_for_ai_json_expansion_from_seed(
        &self, 
        target_dir:            &Path,
        _expected_content_type: &ExpectedContentType,

    ) -> PathBuf {

        // Convert 'token_name' to snake_case
        let snake_token_name = to_snake_case(&self.associated_output_name());

        // Determine the output filename based on custom_id
        // You can customize this as needed, e.g., using token names
        let filename = format!("{}.json", snake_token_name);

        target_dir.to_path_buf().join(filename)
    }
}
