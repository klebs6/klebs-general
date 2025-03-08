// ---------------- [ File: src/workspace_interface.rs ]
crate::ix!();

pub trait BatchWorkspaceInterface
: GetInputFilenameAtIndex
+ GetOutputFilenameAtIndex
+ GetErrorFilenameAtIndex
+ GetMetadataFilenameAtIndex
+ GetDoneDirectory
+ GetTokenExpansionPath
+ GetFailedJsonRepairsDir
+ GetFailedItemsDir
+ GetTextStoragePath
+ GetWorkdir
+ Send
+ Sync
{}

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

pub trait GetTokenExpansionPath {
    fn token_expansion_path(&self,token_name: &CamelCaseTokenWithComment) -> PathBuf;
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
