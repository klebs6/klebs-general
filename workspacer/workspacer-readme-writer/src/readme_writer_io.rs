crate::ix!();

#[derive(Debug,Clone)]
pub struct AiReadmeWriterRequest {
    crate_name:                   String,
    consolidated_crate_interface: ConsolidatedCrateInterface,
    package_authors:              Vec<String>,
    rust_edition:                 String,
    license:                      String,
    repository:                   String,
    possible_categories:          Vec<(String,String)>,
}

impl std::fmt::Display for AiReadmeWriterRequest {

    fn fmt(&self, x: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(x,"ai-toml-writer-request-for-{}",&self.crate_name)
    }
}

impl Named for AiReadmeWriterRequest {

    fn name(&self) -> Cow<'_,str> {
        Cow::Owned(format!("{}-ai-toml-writer-request",&self.crate_name))
    }
}

/// the doc comment here will be used to construct the query
#[derive(AiJsonTemplate,Debug,Clone,Serialize,Deserialize)]
pub struct AiReadmeWriterDesiredOutput {
    full_readme_markdown: String,

    /// the doc comment here will be used to construct the query
    package_description: String,

    /// the doc comment here will be used to construct the query
    package_keywords:    Vec<String>,

    /// the doc comment here will be used to construct the query
    package_categories:  Vec<String>,
}
