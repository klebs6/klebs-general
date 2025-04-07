crate::ix!();

/// Extended ShowFlags with a new `show_items_with_no_data` flag.
/// If `show_items_with_no_data` is `true`, we'll display placeholders:
/// - `<no-data-for-crate>` if a crate is empty
/// - `<no-data-for-file>` if a file grouping is empty (though in the current approach, we never track truly empty files)
/// - `<no-data>` if the entire final output is empty
#[derive(StructOpt,MutGetters,Getters,Builder,Debug)]
#[builder(setter(into))]
#[getset(get="pub",get_mut="pub")]
pub struct ShowFlags {
    /// Path to the crate (or workspace root) you want to show.
    #[structopt(long = "path", parse(from_os_str))]
    path: Option<PathBuf>,

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

    /// Show only test items (skips non-test)
    #[structopt(long = "just-tests")]
    just_tests: bool,

    /// Show only free functions (skips impls, structs, etc.)
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

    /// Group items by the file in which they were found
    #[structopt(long = "group-by-file")]
    group_by_file: bool,

    /// For `crate-tree` subcommand, do NOT merge all crates into one interface
    /// (the new default). If false, merges them all.
    #[structopt(long = "merge-crates")]
    merge_crates: bool,

    /// If set, we show <no-data-for-crate> or <no-data-for-file> or <no-data>
    /// even if a crate or file has no data.
    #[structopt(long = "show-items-with-no-data")]
    show_items_with_no_data: bool,
}

// Provide `From<&ShowFlags>` => `ConsolidationOptions`, so we can do
// `ConsolidationOptions::from(flags)` easily.
impl From<&ShowFlags> for ConsolidationOptions {
    fn from(sf: &ShowFlags) -> Self {
        let mut opts = ConsolidationOptions::new();
        if sf.include_docs {
            opts = opts.with_docs();
        }
        if sf.include_private {
            opts = opts.with_private_items();
        }
        if sf.include_tests {
            opts = opts.with_test_items();
        }
        if sf.include_fn_bodies {
            opts = opts.with_fn_bodies();
        }
        if sf.include_test_bodies {
            opts = opts.with_fn_bodies_in_tests();
        }
        if sf.just_tests {
            opts = opts.with_only_test_items();
        }
        opts.validate();
        opts
    }
}

impl ShowFlags {

    #[tracing::instrument(level="trace", skip(self, cci, crate_name))]
    pub fn build_filtered_grouped_by_file_string(
        &self, 
        cci: &ConsolidatedCrateInterface, 
        crate_name: &str
    ) -> String {
        trace!("Grouping items by file for crate: {}", crate_name);

        // If the entire crate is empty, we won't have any items at all
        let mut out = String::new();
        let mut items_by_file: HashMap<PathBuf, Vec<ConsolidatedItem>> = HashMap::new();

        fn push_item(
            map: &mut HashMap<PathBuf, Vec<ConsolidatedItem>>,
            path: &PathBuf,
            item: ConsolidatedItem,
        ) {
            map.entry(path.clone()).or_insert_with(Vec::new).push(item);
        }

        // 1) Gather items from each category
        for fn_item in cci.fns() {
            let p = fn_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Fn(fn_item.clone()));
        }
        for st_item in cci.structs() {
            let p = st_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Struct(st_item.clone()));
        }
        for en_item in cci.enums() {
            let p = en_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Enum(en_item.clone()));
        }
        for tr_item in cci.traits() {
            let p = tr_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Trait(tr_item.clone()));
        }
        for ta_item in cci.type_aliases() {
            let p = ta_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::TypeAlias(ta_item.clone()));
        }
        for mac_item in cci.macros() {
            let p = mac_item.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Macro(mac_item.clone()));
        }
        for ib in cci.impls() {
            let p = ib.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::ImplBlock(ib.clone()));
        }
        for mo in cci.modules() {
            let p = mo.file_path().clone();
            push_item(&mut items_by_file, &p, ConsolidatedItem::Module(mo.clone()));
        }

        // If cci is truly empty (no items), items_by_file is empty
        if items_by_file.is_empty() {
            // If show_items_with_no_data => show <no-data-for-crate>, else return "".
            if self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            } else {
                return String::new();
            }
        }

        // 2) Sort the file paths so output is deterministic
        let mut file_paths: Vec<_> = items_by_file.keys().cloned().collect();
        file_paths.sort();

        // 3) Build output by file
        for (i, fpath) in file_paths.iter().enumerate() {
            let items = items_by_file.get(fpath).unwrap();
            if items.is_empty() {
                // If the map actually stored an empty vector (which the current code won't),
                // we can handle it:
                if self.show_items_with_no_data {
                    out.push_str(&format!("--- [File: {}] ---\n", fpath.display()));
                    out.push_str("<no-data-for-file>\n\n");
                }
                // else skip
                continue;
            }

            out.push_str(&format!("--- [File: {}] ---\n", fpath.display()));
            // Write each item
            for (j, it) in items.iter().enumerate() {
                out.push_str(&format!("{}", it));
                out.push('\n');
                if j + 1 < items.len() {
                    out.push('\n');
                }
            }
            if i + 1 < file_paths.len() {
                out.push('\n');
            }
        }

        out
    }

    #[tracing::instrument(level="trace", skip(self, cci))]
    pub fn build_filtered_string(
        &self, 
        cci: &ConsolidatedCrateInterface, 
        crate_name: &str
    ) -> String {
        trace!("Applying post-filters in ShowFlags::build_filtered_string");
        let mut out = String::new();

        // If the entire crate is empty (no items in cci), we can short-circuit
        let crate_is_empty = cci.fns().is_empty()
            && cci.structs().is_empty()
            && cci.enums().is_empty()
            && cci.traits().is_empty()
            && cci.type_aliases().is_empty()
            && cci.macros().is_empty()
            && cci.impls().is_empty()
            && cci.modules().is_empty();

        if crate_is_empty {
            if self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            } else {
                return String::new();
            }
        }

        // If user wants to group by file, do that route:
        if *self.group_by_file() {
            return self.build_filtered_grouped_by_file_string(cci, crate_name);
        }

        // If the user wants only a certain category
        if self.just_fns {
            let all_fns = cci.fns();
            if all_fns.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, item) in all_fns.iter().enumerate() {
                out.push_str(&format!("{}", item));
                out.push('\n');
                if i + 1 < all_fns.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_impls {
            let all_impls = cci.impls();
            if all_impls.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, ib) in all_impls.iter().enumerate() {
                out.push_str(&format!("{}", ib));
                out.push('\n');
                if i + 1 < all_impls.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_traits {
            let traits = cci.traits();
            if traits.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, tr) in traits.iter().enumerate() {
                out.push_str(&format!("{}", tr));
                out.push('\n');
                if i + 1 < traits.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_enums {
            let enums = cci.enums();
            if enums.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, en) in enums.iter().enumerate() {
                out.push_str(&format!("{}", en));
                out.push('\n');
                if i + 1 < enums.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_structs {
            let structs = cci.structs();
            if structs.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, st) in structs.iter().enumerate() {
                out.push_str(&format!("{}", st));
                out.push('\n');
                if i + 1 < structs.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_aliases {
            let aliases = cci.type_aliases();
            if aliases.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, ta) in aliases.iter().enumerate() {
                out.push_str(&format!("{}", ta));
                out.push('\n');
                if i + 1 < aliases.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_adts {
            let mut combined: Vec<String> = Vec::new();
            for e in cci.enums() {
                combined.push(format!("{}", e));
            }
            for s in cci.structs() {
                combined.push(format!("{}", s));
            }
            if combined.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, out_str) in combined.iter().enumerate() {
                out.push_str(out_str);
                out.push('\n');
                if i + 1 < combined.len() {
                    out.push('\n');
                }
            }
            return out;
        }
        if self.just_macros {
            let macros = cci.macros();
            if macros.is_empty() && self.show_items_with_no_data {
                return "<no-data-for-crate>\n".to_string();
            }
            for (i, mac) in macros.iter().enumerate() {
                out.push_str(&format!("{}", mac));
                out.push('\n');
                if i + 1 < macros.len() {
                    out.push('\n');
                }
            }
            return out;
        }

        // If again user wants group_by_file
        if self.group_by_file {
            return self.build_filtered_grouped_by_file_string(cci, "unknown-crate");
        }

        // Otherwise, print the entire consolidated interface in one go
        // i.e. all items if no narrower filter
        let all = format!("{}", cci);
        if all.trim().is_empty() && self.show_items_with_no_data {
            return "<no-data-for-crate>\n".to_string();
        }
        out.push_str(&all);
        out
    }
}


