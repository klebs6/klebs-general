crate::ix!();

#[derive(Builder,Getters,Debug,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct FileFilterConfig {
    // If the file is bigger than this in bytes, we might skip or fallback. 
    // You can expand logic as desired.
    max_file_size_bytes: Option<u64>,
    // If you have other toggles, like "include_filenames_in_prompt" etc., 
    // add them here, using getset + derive_builder.
}

impl Default for FileFilterConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: Some(512_000), // ~512KB
        }
    }
}
