// ---------------- [ File: workspacer-show/src/options.rs ]
crate::ix!();

/// Extended ShowFlags with a new `show_items_with_no_data` flag.
/// If `show_items_with_no_data` is `true`, we'll display placeholders:
/// - `<no-data-for-crate>` if a crate is empty
/// - `<no-data-for-file>` if a file grouping is empty (though in the current approach, we never track truly empty files)
/// - `<no-data>` if the entire final output is empty
#[derive(Clone,StructOpt,Setters,MutGetters,Getters,Builder,Debug)]
#[builder(setter(into))]
#[getset(get="pub",get_mut="pub",set="pub")]
pub struct ShowFlags {
    /// Path to the crate (or workspace root) you want to show.
    #[structopt(long = "path", parse(from_os_str))]
    path: Option<PathBuf>,

    /// Include private items
    #[structopt(long = "include-private")]
    #[builder(default="false")]
    include_private: bool,

    /// Include doc items
    #[structopt(long = "include-docs")]
    #[builder(default="false")]
    include_docs: bool,

    /// Include test items
    #[structopt(long = "include-tests")]
    #[builder(default="false")]
    include_tests: bool,

    /// Include function bodies
    #[structopt(long = "include-fn-bodies")]
    #[builder(default="false")]
    include_fn_bodies: bool,

    /// Include test function bodies
    #[structopt(long = "include-test-bodies")]
    #[builder(default="false")]
    include_test_bodies: bool,

    /// Show only test items (skips non-test)
    #[structopt(long = "just-tests")]
    #[builder(default="false")]
    just_tests: bool,

    /// Show only free functions (skips impls, structs, etc.)
    #[structopt(long = "just-fns")]
    #[builder(default="false")]
    just_fns: bool,

    /// Show only impl blocks
    #[structopt(long = "just-impls")]
    #[builder(default="false")]
    just_impls: bool,

    /// Show only traits
    #[structopt(long = "just-traits")]
    #[builder(default="false")]
    just_traits: bool,

    /// Show only enums
    #[structopt(long = "just-enums")]
    #[builder(default="false")]
    just_enums: bool,

    /// Show only structs
    #[structopt(long = "just-structs")]
    #[builder(default="false")]
    just_structs: bool,

    /// Show only type aliases
    #[structopt(long = "just-aliases")]
    #[builder(default="false")]
    just_aliases: bool,

    /// Show only ADTs (enums + structs)
    #[structopt(long = "just-adts")]
    #[builder(default="false")]
    just_adts: bool,

    /// Show only macros
    #[structopt(long = "just-macros")]
    #[builder(default="false")]
    just_macros: bool,

    /// Group items by the file in which they were found
    #[structopt(long = "group-by-file")]
    #[builder(default="true")]
    group_by_file: bool,

    /// For `crate-tree` subcommand, do NOT merge all crates into one interface
    /// (the new default). If false, merges them all.
    #[structopt(long = "merge-crates")]
    #[builder(default="false")]
    merge_crates: bool,

    /// If set, we show <no-data-for-crate> or <no-data-for-file> or <no-data>
    /// even if a crate or file has no data.
    #[structopt(long = "show-items-with-no-data")]
    #[builder(default="false")]
    show_items_with_no_data: bool,
}

impl From<&ShowFlags> for ConsolidationOptions {
    /// Helper to map ShowFlags into a `ConsolidationOptions`.
    fn from(opts: &ShowFlags) -> ConsolidationOptions {
        let mut c = ConsolidationOptions::new();
        if *opts.include_docs() {
            c = c.with_docs();
        }
        if *opts.include_private() {
            c = c.with_private_items();
        }
        if *opts.include_tests() {
            c = c.with_test_items();
        }
        if *opts.include_fn_bodies() {
            c = c.with_fn_bodies();
        }
        if *opts.include_test_bodies() {
            c = c.with_fn_bodies_in_tests();
        }
        if *opts.just_tests() {
            c = c.with_only_test_items();
        }
        c.validate();
        c
    }
}
