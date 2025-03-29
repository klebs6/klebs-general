// ---------------- [ File: workspacer-cli/src/show.rs ]
crate::ix!();

/// Options for `ws show` (include or exclude certain items).
///
/// Show consolidated interface with various filtering flags
#[derive(Debug, StructOpt)]
pub struct ShowSubcommand {
    /// Include private items
    #[structopt(long = "include-private")]
    include_private: bool,

    /// Include doc items
    #[structopt(long = "include-docs")]
    include_docs: bool,

    /// Include test items
    #[structopt(long = "include-tests")]
    include_tests: bool,

    /// Include function bodies
    #[structopt(long = "include-fn-bodies")]
    include_fn_bodies: bool,

    /// Include test function bodies
    #[structopt(long = "include-test-bodies")]
    include_test_bodies: bool,

    /// Show only test items
    #[structopt(long = "just-tests")]
    just_tests: bool,

    /// Show only free functions
    #[structopt(long = "just-fns")]
    just_fns: bool,

    /// Show only impl blocks
    #[structopt(long = "just-impls")]
    just_impls: bool,

    /// Show only traits
    #[structopt(long = "just-traits")]
    just_traits: bool,

    /// Show only enums
    #[structopt(long = "just-enums")]
    just_enums: bool,

    /// Show only structs
    #[structopt(long = "just-structs")]
    just_structs: bool,

    /// Show only type aliases
    #[structopt(long = "just-aliases")]
    just_aliases: bool,

    /// Show only ADTs (enums + structs)
    #[structopt(long = "just-adts")]
    just_adts: bool,

    /// Show only macros
    #[structopt(long = "just-macros")]
    just_macros: bool,
}

impl ShowSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!();
    }
}
