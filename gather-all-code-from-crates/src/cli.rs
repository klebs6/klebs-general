crate::ix!();

#[derive(Builder,Getters,StructOpt,Debug)]
#[structopt(name = "gather-all-code-from-crates")]
#[builder(setter(into))]
pub struct Cli {
    /// One or more crate directories to scan. If none, current dir is used.
    #[structopt(parse(from_os_str), short="c", long="crate")]
    #[getset(get = "pub")]
    crates: Vec<PathBuf>,

    /// Include test code?
    #[structopt(long="include-tests")]
    #[getset(get = "pub")]
    #[builder(default = "false")]
    include_tests: bool,

    /// Single out one test block by name.
    #[structopt(long="single-test")]
    #[getset(get = "pub")]
    #[builder(default)]
    single_test_name: Option<String>,

    /// Omit private functions?
    #[structopt(long="omit-private")]
    #[getset(get = "pub")]
    #[builder(default = "false")]
    omit_private: bool,

    /// Omit function bodies?
    #[structopt(long="omit-bodies")]
    #[getset(get = "pub")]
    #[builder(default = "false")]
    omit_bodies: bool,

    /// Only include one function body by name.
    #[structopt(long="single-function")]
    #[getset(get = "pub")]
    #[builder(default)]
    single_function_name: Option<String>,

    /// Exclude specified files (by relative path).
    #[structopt(long="exclude-file")]
    #[getset(get = "pub")]
    #[builder(default)]
    excluded_files: Vec<String>,

    /// Exclude main file (like src/lib.rs)?
    #[structopt(long="exclude-main-file")]
    #[getset(get = "pub")]
    #[builder(default = "false")]
    exclude_main_file: bool,

    /// Remove doc comments from functions
    #[structopt(long="remove-doc-comments")]
    #[getset(get = "pub")]
    #[builder(default = "false")]
    remove_doc_comments: bool,
}
