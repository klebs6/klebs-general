crate::ix!();

#[derive(AiJsonTemplate, SaveLoad, Builder, Getters, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
#[builder(setter(into))]
pub struct AiFileFilterDesiredOutput {
    // The path again for reference:
    file_path: String,

    // The newly filtered text:
    filtered_text: String,
}

impl Named for AiFileFilterDesiredOutput {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        // We'll name it e.g. "filtered-FILENAME", or just the path
        std::borrow::Cow::Owned(format!("filtered-{}", self.file_path()))
    }
}
