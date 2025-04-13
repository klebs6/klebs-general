// ---------------- [ File: workspacer-file-filter/src/workspacer_file_filter_desired_output.rs ]
crate::ix!();

#[derive(
    AiJsonTemplate, 
    SaveLoad, 
    Builder, 
    Getters, 
    Debug, 
    Clone, 
    Serialize, 
    Deserialize
)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct AiFileFilterDesiredOutput {

    // This is the full path of the file we are filtering. It should be full and the *exact same* as the one we sent you.
    file_path: String,

    // This string is the filtered text, we would like you to generate.
    filtered_text: String,
}

impl AiFileFilterDesiredOutput {

    pub fn snake_path_stem(&self) -> String {
        filename_to_snake(&self.file_path)
    }
}

impl Named for AiFileFilterDesiredOutput {

    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Owned(format!("filtered_{}", self.snake_path_stem()))
    }
}
