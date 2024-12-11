crate::ix!();

// Global configuration in JSON, merging with CLI args.
#[derive(Default,Debug, Clone, Deserialize, Builder, Getters, Setters)]
#[builder(setter(into))]
#[getset(get = "pub", set = "pub")]
pub struct AstFilterCriteria {
    #[serde(default)]
    #[builder(default)]
    include_tests: bool,

    #[serde(default)]
    #[builder(default)]
    single_test_name: Option<String>,

    #[serde(default)]
    #[builder(default)]
    omit_private: bool,

    #[serde(default)]
    #[builder(default)]
    omit_bodies: bool,

    #[serde(default)]
    #[builder(default)]
    single_function_name: Option<String>,

    #[serde(default)]
    #[builder(default)]
    excluded_files: Vec<String>,

    #[serde(default)]
    #[builder(default)]
    exclude_main_file: bool,

    #[serde(default)]
    #[builder(default)]
    remove_doc_comments: bool,
}
