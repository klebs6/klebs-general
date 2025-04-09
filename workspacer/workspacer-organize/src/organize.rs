crate::ix!();

/// Organize a set of `LosslessFile` inputs into a mapping of `(filename -> file_content)`.
///
/// **High-level rules**:
/// 1) Each resulting file has exactly *one* primary "public" item (fn, struct, trait, etc.).
/// 2) If that public item is a struct, we also gather all *impl blocks* associated with it
///    (including trait impls) from across all input files, plus any test items specifically for them.
/// 3) We also gather "test items" (#[cfg(test)] or in test modules) that appear in the same original
///    file and presumably relate to that main item. (In practice, we do a simpler approach: any test
///    item in the same file is appended, but you can refine if needed.)
/// 4) The filename is derived from the public item's name in robust snake_case. We then ensure no collisions
///    by appending `_2`, `_3`, etc. as needed.
/// 5) If an item has no valid name or we can't parse it properly, we fallback to a stable `untitled_item_<N>` name
///    that does *not* contain invalid characters. We do *not* produce `<unknown>`.
///
/// **Usage**:
/// ```ignore
/// let organized = organize_lossless_files(&all_lossless_files);
/// for (path, content) in organized {
///     // write to filesystem
/// }
/// ```
pub fn organize_lossless_files(lossless_files: &[LosslessFile]) -> HashMap<PathBuf,String> {
    trace!("Starting organize_lossless_files with {} input files", lossless_files.len());

    // (1) Gather all items from all files
    let all_items = gather_all_items(lossless_files);

    // (2) Identify all public items
    let public_items = gather_public_items(&all_items);

    // (3) Build a map from "struct name" => all impl blocks referencing that struct.
    //     This helps us group "pub struct X" and its trait impls or inherent impls.
    let struct_impl_map = group_impl_blocks_by_subject(&all_items);

    // (4) We'll produce the final mapping (filename -> file_content).
    let mut result_map: HashMap<PathBuf,String> = HashMap::new();
    let mut used_filenames: HashSet<String> = HashSet::new();

    // (5) For each public item, build or find a unique .rs file.
    for pub_entry in public_items {
        // Derive a "base" for the file name
        let base_name = derive_base_filename(&pub_entry.li.item);
        let unique_path = make_unique_filename(&base_name, &mut used_filenames);
        info!("Creating new file for public item: {} => {:?}", base_name, unique_path);

        let mut file_text = String::new();
        append_item_snippet(&mut file_text, &pub_entry.li.original_snippet());

        // If it's a public struct, gather all matching impl blocks
        if let CStruct(struct_ci) = &pub_entry.li.item {
            let sname = struct_ci
                .item()
                .name()
                .unwrap_or_default()
                .to_lowercase();
            // `derive_base_filename` would have handled empties, but let's be safe:
            if !sname.is_empty() {
                if let Some(impl_entries) = struct_impl_map.get(&sname) {
                    for imp in impl_entries {
                        trace!("Adding impl block snippet for struct={}", sname);
                        append_item_snippet(&mut file_text, &imp.li.original_snippet());
                    }
                }
            }
        }

        // Also gather "test items" from the same file if relevant
        let test_snippets = gather_test_items_for_public_item(lossless_files, &pub_entry);
        for snippet in test_snippets {
            append_item_snippet(&mut file_text, &snippet);
        }

        result_map.insert(unique_path, file_text);
    }

    result_map
}

// --------------------------------------------------------------------------
//  Subroutines
// --------------------------------------------------------------------------

/// Gathers all items from all `LosslessFile`s into a vector.
fn gather_all_items(lossless_files: &[LosslessFile]) -> Vec<AllItemRecord> {
    trace!("Gathering all items from {} files...", lossless_files.len());
    let mut items = Vec::new();
    for lf in lossless_files {
        for (idx, li) in lf.items().iter().enumerate() {
            items.push(AllItemRecord {
                file_path: lf.file_path().clone(),
                item_index: idx,
                li: li.clone(),
            });
        }
    }
    items
}

/// Among all items, return only the "public" ones (that is, the ones
/// `is_public_item(...)` calls public).
fn gather_public_items(all_items: &[AllItemRecord]) -> Vec<AllItemRecord> {
    let mut v = Vec::new();
    for rec in all_items {
        if is_public_item(&rec.li.item) {
            debug!("Public item found in file={:?}, idx={}", rec.file_path, rec.item_index);
            v.push(rec.clone());
        }
    }
    v
}

/// Groups all impl blocks by the type/struct name they reference. This
/// includes both trait impls and inherent impls. We strip generics in a robust manner.
/// e.g. "impl SomeTrait<T> for MyType<U> { ... }" => subject "mytype".
/// e.g. "impl MyStruct { ... }" => subject "mystruct".
fn group_impl_blocks_by_subject(all_items: &[AllItemRecord]) -> HashMap<String,Vec<AllItemRecord>> {
    let mut map: HashMap<String,Vec<AllItemRecord>> = HashMap::new();

    for rec in all_items {
        if let CImplBlock(ib) = &rec.li.item {
            if let Some(subject) = extract_impl_subject(ib.signature_text()) {
                map.entry(subject).or_default().push(rec.clone());
            }
        }
    }
    map
}

/// Attempt to parse the "impl subject" from a line like `"impl SomeTrait for MyStruct<T>"`.
/// We do so robustly by ignoring generics, trimming spaces, etc.
fn extract_impl_subject(sig: &str) -> Option<String> {
    let trimmed = sig.trim();
    // e.g. "impl Something<T> for MyType<U> where ..."
    // Remove leading "impl"
    let after_impl = match trimmed.strip_prefix("impl") {
        Some(s) => s,
        None => {
            debug!("extract_impl_subject: signature didn't start with impl => {:?}", sig);
            return None;
        }
    };
    let after_impl = after_impl.trim();
    // If there's " for ", we handle trait impl:
    if let Some(pos) = after_impl.find(" for ") {
        let after_for = &after_impl[pos+4..];
        // e.g. " MyType<U> where ..."
        // strip generics: take up to first '<', if any
        let sub = after_for.split('<').next().unwrap_or(after_for);
        // also remove any "where" or trailing braces
        let pieces: Vec<_> = sub.trim().split_whitespace().collect();
        if let Some(last) = pieces.last() {
            let clean = last.replace(';',"").replace('{',"").replace('}',"").trim().to_string();
            let base = to_snake_case(&clean);
            return Some(base);
        }
        None
    } else {
        // e.g. "MyType { ... }" => inherent
        let sub = after_impl.split('<').next().unwrap_or(after_impl);
        let pieces: Vec<_> = sub.trim().split_whitespace().collect();
        if let Some(last) = pieces.last() {
            let clean = last.replace(';',"").replace('{',"").replace('}',"").trim().to_string();
            let base = to_snake_case(&clean);
            return Some(base);
        }
        None
    }
}

/// For a given primary public item, gather test items from the same file that might be relevant.
/// Current approach: gather any item in the same file that is "test_item" => return snippet.
fn gather_test_items_for_public_item(
    all_files: &[LosslessFile],
    pub_rec: &AllItemRecord
) -> Vec<String> {
    let mut out = Vec::new();
    // Find the original LosslessFile object
    let lf_opt = all_files.iter().find(|lf| lf.file_path() == &pub_rec.file_path);
    if lf_opt.is_none() {
        return out;
    }
    let lf = lf_opt.unwrap();
    // Gather all items in that file
    for li in lf.items() {
        let item = &li.item;
        // skip the item if it's the same as the public item
        if std::ptr::eq(item, &pub_rec.li.item) {
            continue;
        }
        // check if it's test
        if is_test_item(item) {
            debug!("Found test item in same file => appended");
            out.push(li.original_snippet().to_string());
        }
    }
    out
}

/// Decide if an item is test-only. We'll check if it's in a test module or
/// has #[cfg(test)]. This is a robust approach that doesn't break.
fn is_test_item(item: &ConsolidatedItem) -> bool {
    match item {
        CFn(ci)        => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CStruct(ci)    => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CEnum(ci)      => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CTrait(ci)     => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CTypeAlias(ci) => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CMacro(ci)     => {
            is_in_test_module(ci.item().syntax().clone()) || has_cfg_test_attr(ci.item().syntax())
        }
        CImplBlock(_)  => false, // If the impl block is in a test mod, we might say yes, but let's skip
        CModule(_)     => false, // we skip
        CMockTest(_)   => true,
    }
}

/// Derive a base filename from a public item.  We never produce `<unknown>`.
/// Instead, if there's no recognized name or it's invalid, we produce `untitled_item_<N>`.
fn derive_base_filename(item: &ConsolidatedItem) -> String {
    let raw_name = match item {
        CFn(ci)        => ci.item().name().unwrap_or_default(),
        CStruct(ci)    => ci.item().name().unwrap_or_default(),
        CEnum(ci)      => ci.item().name().unwrap_or_default(),
        CTrait(ci)     => ci.item().name().unwrap_or_default(),
        CTypeAlias(ci) => ci.item().name().unwrap_or_default(),
        CMacro(ci)     => ci.item().name().unwrap_or_default(),
        CImplBlock(ib) => {
            // "impl_xxx_for_yyy"
            let sig = ib.signature_text();
            match parse_impl_filename_fragment(&sig) {
                Some(s) => s,
                None => "impl_standalone".to_string(),
            }
        }
        CModule(m)     => m.mod_name().clone(),
        CMockTest(s)   => s.clone(),
    };

    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        // fallback
        // We'll produce a placeholder. The calling code ensures uniqueness with
        // an incremental index if needed. So let's just say "untitled_item".
        return "untitled_item".to_string();
    }

    // convert to robust snake_case
    let snake = to_snake_case(trimmed);

    // if that is empty or invalid, fallback
    if snake.is_empty() {
        "untitled_item".to_string()
    } else {
        snake
    }
}

/// Parse something like "impl Foo" or "impl SomeTrait for Foo" => produce "impl_foo_for_bar"
/// in a robust manner. If it fails, None.
fn parse_impl_filename_fragment(sig: &str) -> Option<String> {
    let s = sig.trim().trim_start_matches("impl").trim();
    if s.is_empty() {
        return None;
    }
    // if there's "for"
    if let Some(for_pos) = s.find(" for ") {
        let left = &s[..for_pos];
        let right= &s[for_pos+4..];
        // remove generics
        let left_name  = left.split('<').next().unwrap_or(left).trim();
        let right_name = right.split('<').next().unwrap_or(right).trim();
        // snake them:
        let left_snake = to_snake_case(left_name);
        let right_snake= to_snake_case(right_name);
        let combined = format!("impl_{}_for_{}", left_snake, right_snake);
        if combined.contains("impl__for_") { // means something was empty
            return None;
        }
        Some(combined)
    } else {
        // inherent
        let base = s.split('<').next().unwrap_or(s).trim();
        let snake = to_snake_case(base);
        if snake.is_empty() {
            None
        } else {
            Some(format!("impl_for_{}", snake))
        }
    }
}

/// Make a final unique `.rs` filename, appending `_2`, `_3`, etc. if needed.
fn make_unique_filename(base: &str, used: &mut HashSet<String>) -> PathBuf {
    let mut candidate = format!("{}.rs", base);
    let mut idx = 2;
    while used.contains(&candidate) {
        candidate = format!("{}_{}.rs", base, idx);
        idx += 1;
    }
    used.insert(candidate.clone());
    PathBuf::from(candidate)
}

/// Append an item snippet to `file_text`, ensuring it ends with a newline.
fn append_item_snippet(file_text: &mut String, snippet: &str) {
    if !file_text.ends_with('\n') {
        file_text.push('\n');
    }
    file_text.push_str(snippet);
    if !file_text.ends_with('\n') {
        file_text.push('\n');
    }
}

// --------------------------------------------------------------------------
//  Supporting data struct
// --------------------------------------------------------------------------

#[derive(Debug,Clone)]
struct AllItemRecord {
    file_path: PathBuf,
    item_index: usize,
    li: LosslessItem,
}

#[cfg(test)]
mod test_item_reorganizer_subroutines {
    use super::*;
    use tracing::{trace,debug,info,warn,error};
    use tracing_test::traced_test;

    // We'll also need some mock structures and minimal stubs for LosslessFile, LosslessItem, etc.
    // to exercise these subroutines. For real code, adapt these to your actual test harness.

    // A helper for building a simple LosslessFile with items for testing:
    fn make_lossless_file_with_items(
        path: &str,
        items: Vec<LosslessItem>,
    ) -> LosslessFile {
        LosslessFile {
            file_path: PathBuf::from(path),
            original_text: String::new(), // we won't rely on the full text for these subroutine tests
            items,
            interstitials: vec![],
            layout: vec![],
        }
    }

    fn dummy_ci_struct(name: &str) -> ConsolidatedItem {
        // Normally you'd build a real `ast::Struct`, but for these subroutines we can mock:
        let ci = CrateInterfaceItem::new_for_test(
            mock_ast_struct(name),
            None,
            None,
            None,
            None
        );
        ConsolidatedItem::Struct(ci)
    }

    fn dummy_ci_fn(name: &str) -> ConsolidatedItem {
        let ci = CrateInterfaceItem::new_for_test(
            mock_ast_fn(name),
            None,
            None,
            None,
            None
        );
        ConsolidatedItem::Fn(ci)
    }

    fn dummy_ci_impl(signature_text: &str) -> ConsolidatedItem {
        let ib = ImplBlockInterface::new_for_test(
            None,
            None,
            signature_text.to_string(),
            vec![],
            vec![]
        );
        ConsolidatedItem::ImplBlock(ib)
    }

    fn dummy_ci_other(name: &str) -> ConsolidatedItem {
        // something else, e.g. trait or enum
        let ci = CrateInterfaceItem::new_for_test(
            mock_ast_trait(name),
            None,
            None,
            None,
            None
        );
        ConsolidatedItem::Trait(ci)
    }

    // Minimal mock ast node placeholders for the CrateInterfaceItem
    #[derive(Debug,Clone)]
    struct MockStruct { name: String }
    impl workspacer_syntax::GenerateSignature for MockStruct {
        fn generate_signature_with_opts(&self, _opts: &SignatureOptions) -> String {
            format!("pub struct {}", self.name)
        }
    }
    impl workspacer_syntax::RehydrateFromSignature for MockStruct {
        fn rehydrate_from_signature(_signature_source: &str) -> Option<Self> {
            None
        }
    }
    fn mock_ast_struct(name: &str) -> MockStruct {
        MockStruct { name: name.to_string() }
    }

    #[derive(Debug,Clone)]
    struct MockFn { name: String }
    impl workspacer_syntax::GenerateSignature for MockFn {
        fn generate_signature_with_opts(&self, _opts: &SignatureOptions) -> String {
            format!("pub fn {}()", self.name)
        }
    }
    impl workspacer_syntax::RehydrateFromSignature for MockFn {
        fn rehydrate_from_signature(_signature_source: &str) -> Option<Self> {
            None
        }
    }
    fn mock_ast_fn(name: &str) -> MockFn {
        MockFn { name: name.to_string() }
    }

    #[derive(Debug,Clone)]
    struct MockTrait { name: String }
    impl workspacer_syntax::GenerateSignature for MockTrait {
        fn generate_signature_with_opts(&self, _opts: &SignatureOptions) -> String {
            format!("pub trait {}", self.name)
        }
    }
    impl workspacer_syntax::RehydrateFromSignature for MockTrait {
        fn rehydrate_from_signature(_signature_source: &str) -> Option<Self> {
            None
        }
    }
    fn mock_ast_trait(name: &str) -> MockTrait {
        MockTrait { name: name.to_string() }
    }

    // ------------------------------------------------------------------------------
    //  test_gather_all_items
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_gather_all_items {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_empty_list() {
            info!("Testing gather_all_items with no input files => empty output");
            let result = gather_all_items(&[]);
            assert!(result.is_empty(), "No files => no items");
        }

        #[traced_test]
        fn test_single_file_multiple_items() {
            info!("One file with 3 items => gather_all_items => length=3");
            let lf = make_lossless_file_with_items(
                "file1.rs",
                vec![
                    LosslessItem {
                        item: dummy_ci_struct("Apple"),
                        original_snippet: "pub struct Apple {}".into()
                    },
                    LosslessItem {
                        item: dummy_ci_fn("do_stuff"),
                        original_snippet: "pub fn do_stuff() {}".into()
                    },
                    LosslessItem {
                        item: dummy_ci_impl("impl Apple {}"),
                        original_snippet: "impl Apple {}".into()
                    },
                ]
            );
            let result = gather_all_items(&[lf]);
            assert_eq!(result.len(), 3, "Should gather 3 items");
            assert_eq!(result[0].file_path, PathBuf::from("file1.rs"));
            // etc. check the last item is an impl etc.
        }

        #[traced_test]
        fn test_multiple_files() {
            info!("Gather items from multiple files");
            let lf1 = make_lossless_file_with_items("f1.rs", vec![LosslessItem {
                item: dummy_ci_struct("Foo"),
                original_snippet: "pub struct Foo;".into()
            }]);
            let lf2 = make_lossless_file_with_items("f2.rs", vec![
                LosslessItem {
                    item: dummy_ci_fn("alpha"),
                    original_snippet: "pub fn alpha() {}".into()
                },
                LosslessItem {
                    item: dummy_ci_other("Omega"),
                    original_snippet: "pub trait Omega {}".into()
                }
            ]);
            let all = gather_all_items(&[lf1, lf2]);
            assert_eq!(all.len(), 3);
            let paths: Vec<_> = all.iter().map(|r| r.file_path.clone()).collect();
            // check f1.rs or f2.rs
            assert!(paths.contains(&PathBuf::from("f1.rs")));
            assert!(paths.contains(&PathBuf::from("f2.rs")));
        }
    }

    // ------------------------------------------------------------------------------
    //  test_gather_public_items
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_gather_public_items {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_no_public() {
            info!("No public items => gather_public_items => empty");
            let all = vec![
                AllItemRecord {
                    file_path: "file.rs".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: {
                            // Suppose we have a private fn or struct
                            // We'll just not put 'pub' in the snippet or we rely on is_public_item to be false
                            let ci = CrateInterfaceItem::new_for_test(mock_ast_fn("private_fn"), None, None, None, None);
                            ConsolidatedItem::Fn(ci)
                        },
                        original_snippet: "fn private_fn() {}".into(),
                    },
                }
            ];
            let pub_items = gather_public_items(&all);
            assert!(pub_items.is_empty(), "No public => empty");
        }

        #[traced_test]
        fn test_some_public() {
            info!("Mix of public and private => only public returned");
            let all = vec![
                AllItemRecord {
                    file_path: "f.rs".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: dummy_ci_fn("alpha"), // we assume it's 'pub fn alpha'
                        original_snippet: "pub fn alpha() {}".to_string()
                    }
                },
                AllItemRecord {
                    file_path: "f.rs".into(),
                    item_index: 1,
                    li: LosslessItem {
                        item: {
                            let ci = CrateInterfaceItem::new_for_test(mock_ast_fn("beta"), None, None, None, None);
                            // pretend it's private
                            ConsolidatedItem::Fn(ci)
                        },
                        original_snippet: "fn beta() {}".into()
                    }
                },
            ];
            let pub_items = gather_public_items(&all);
            assert_eq!(pub_items.len(), 1, "Only alpha is public");
            assert_eq!(pub_items[0].li.original_snippet, "pub fn alpha() {}");
        }
    }

    // ------------------------------------------------------------------------------
    //  test_group_impl_blocks_by_subject
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_group_impl_blocks_by_subject {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_empty() {
            info!("No items => group_impl_blocks_by_subject => empty map");
            let map = group_impl_blocks_by_subject(&[]);
            assert!(map.is_empty(), "Empty input => empty map");
        }

        #[traced_test]
        fn test_only_non_impl_items() {
            info!("If we have no impl blocks => map is empty");
            let recs = vec![
                AllItemRecord {
                    file_path: "f1".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: dummy_ci_fn("something"),
                        original_snippet: "pub fn something(){}".into()
                    }
                }
            ];
            let map = group_impl_blocks_by_subject(&recs);
            assert!(map.is_empty());
        }

        #[traced_test]
        fn test_single_inherent_impl() {
            info!("One impl MyStruct => subject key is 'my_struct'");
            let recs = vec![
                AllItemRecord {
                    file_path: "f1".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl MyStruct { /* ... */ }"),
                        original_snippet: "impl MyStruct {}".into()
                    }
                }
            ];
            let map = group_impl_blocks_by_subject(&recs);
            assert_eq!(map.len(), 1);
            let (subject, items) = map.iter().next().unwrap();
            assert_eq!(subject, "my_struct");
            assert_eq!(items.len(), 1);
        }

        #[traced_test]
        fn test_trait_impl_for_type() {
            info!("impl SomeTrait for Apple => subject=apple");
            let recs = vec![
                AllItemRecord {
                    file_path: "f2".into(),
                    item_index: 2,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl SomeTrait for Apple {}"),
                        original_snippet: "impl SomeTrait for Apple {}".into()
                    }
                }
            ];
            let map = group_impl_blocks_by_subject(&recs);
            assert_eq!(map.len(), 1);
            assert!(map.contains_key("apple"));
        }

        #[traced_test]
        fn test_multiple_impls_for_same_subject() {
            info!("Multiple impls for 'Kiwi'");
            let recs = vec![
                AllItemRecord {
                    file_path: "f".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl Kiwi {}"),
                        original_snippet: "impl Kiwi {}".into()
                    }
                },
                AllItemRecord {
                    file_path: "f".into(),
                    item_index: 1,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl AnotherTrait for Kiwi {}"),
                        original_snippet: "impl AnotherTrait for Kiwi {}".into()
                    }
                }
            ];
            let map = group_impl_blocks_by_subject(&recs);
            assert_eq!(map.len(), 1);
            let items = map.get("kiwi").unwrap();
            assert_eq!(items.len(), 2);
        }

        #[traced_test]
        fn test_different_subjects() {
            info!("Impls referencing different subjects => multiple map keys");
            let recs = vec![
                AllItemRecord {
                    file_path: "f".into(),
                    item_index: 0,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl Pear {}"),
                        original_snippet: "impl Pear {}".into()
                    }
                },
                AllItemRecord {
                    file_path: "f".into(),
                    item_index: 1,
                    li: LosslessItem {
                        item: dummy_ci_impl("impl TraitX for Mango {}"),
                        original_snippet: "impl TraitX for Mango {}".into()
                    }
                }
            ];
            let map = group_impl_blocks_by_subject(&recs);
            assert_eq!(map.len(), 2);
            assert!(map.contains_key("pear"));
            assert!(map.contains_key("mango"));
        }
    }

    // ------------------------------------------------------------------------------
    //  test_extract_impl_subject
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_extract_impl_subject {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_inherent_simple() {
            info!("extract_impl_subject => 'impl Foo' => subject=foo");
            let subj = extract_impl_subject("impl Foo {}").unwrap();
            assert_eq!(subj, "foo");
        }

        #[traced_test]
        fn test_inherent_generics() {
            info!("impl Foo<T> => subject=foo");
            let subj = extract_impl_subject("impl Foo<T>").unwrap();
            assert_eq!(subj, "foo");
        }

        #[traced_test]
        fn test_trait_impl() {
            info!("impl SomeTrait for Bar => subject=bar");
            let subj = extract_impl_subject("impl SomeTrait for Bar").unwrap();
            assert_eq!(subj, "bar");
        }

        #[traced_test]
        fn test_trait_impl_generics() {
            info!("impl SomeTrait<Z> for Bar<T> => subject=bar");
            let subj = extract_impl_subject("impl SomeTrait<Z> for Bar<T> {}").unwrap();
            assert_eq!(subj, "bar");
        }

        #[traced_test]
        fn test_trim_extras() {
            info!("We ignore trailing braces, semicolons, or 'where' etc.");
            let subj = extract_impl_subject("impl X where X:Clone { }").unwrap();
            assert_eq!(subj, "x");
        }

        #[traced_test]
        fn test_no_impl_prefix() {
            info!("If string doesn't start with 'impl', we return None");
            let s = extract_impl_subject("notanimpl Something {}");
            assert!(s.is_none());
        }

        #[traced_test]
        fn test_empty_after_impl() {
            info!("'impl' with nothing => None");
            let s = extract_impl_subject("impl   ");
            assert!(s.is_none());
        }
    }

    // ------------------------------------------------------------------------------
    //  test_gather_test_items_for_public_item
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_gather_test_items_for_public_item {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_no_test_items_in_file() {
            info!("No test items => gather_test_items => empty");
            let main_pub = AllItemRecord {
                file_path: "file.rs".into(),
                item_index: 0,
                li: LosslessItem {
                    item: dummy_ci_fn("public_fn"),
                    original_snippet: "pub fn public_fn(){}".into()
                }
            };
            let lf = make_lossless_file_with_items(
                "file.rs",
                vec![main_pub.li.clone()]
            );
            let out = gather_test_items_for_public_item(&[lf], &main_pub);
            assert!(out.is_empty());
        }

        #[traced_test]
        fn test_one_test_item_in_same_file() {
            info!("One test item => gather_test_items => returns snippet");
            let pub_item = AllItemRecord {
                file_path: "f".into(),
                item_index: 0,
                li: LosslessItem {
                    item: dummy_ci_fn("do_things"),
                    original_snippet: "pub fn do_things() {}".into()
                }
            };
            // We'll mark next item as a test item by some approach
            let test_li = LosslessItem {
                item: {
                    // We'll just pretend it's a test item => mock a function with snippet
                    // and handle in is_test_item?
                    ConsolidatedItem::MockTest("fn test_do_things() {}".to_string())
                },
                original_snippet: "#[cfg(test)] fn test_do_things() {}".into()
            };
            let lf = make_lossless_file_with_items(
                "f",
                vec![
                    pub_item.li.clone(),
                    test_li.clone(),
                ]
            );
            let result = gather_test_items_for_public_item(&[lf], &pub_item);
            assert_eq!(result.len(), 1);
            assert_eq!(result[0], "#[cfg(test)] fn test_do_things() {}");
        }

        #[traced_test]
        fn test_ignores_other_files() {
            info!("Test items in a different file => not returned");
            let main_item = AllItemRecord {
                file_path: "a.rs".into(),
                item_index: 0,
                li: LosslessItem {
                    item: dummy_ci_fn("xxx"),
                    original_snippet: "pub fn xxx(){}".into()
                }
            };
            let lf_a = make_lossless_file_with_items("a.rs", vec![main_item.li.clone()]);
            let test_li = LosslessItem {
                item: ConsolidatedItem::MockTest("fn test_xxx(){}".to_string()),
                original_snippet: "fn test_xxx(){}".to_string()
            };
            let lf_b = make_lossless_file_with_items("b.rs", vec![test_li]);
            let r = gather_test_items_for_public_item(&[lf_a, lf_b], &main_item);
            assert!(r.is_empty(), "Test item is in a different file => skip");
        }
    }

    // ------------------------------------------------------------------------------
    //  test_is_test_item
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_is_test_item {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_mock_test_variant() {
            info!("Any MockTest => is_test_item => true");
            let c = ConsolidatedItem::MockTest("something".to_string());
            assert!(is_test_item(&c));
        }

        #[traced_test]
        fn test_normal_fn_no_cfg_test() {
            info!("Normal fn => not test");
            let c = dummy_ci_fn("regular");
            assert!(!is_test_item(&c));
        }

        #[traced_test]
        fn test_impl_block() {
            info!("Impl block => we skip => false");
            let c = dummy_ci_impl("impl Something");
            assert!(!is_test_item(&c));
        }

        // In real usage, you'd have an actual function node with #[cfg(test)] or a test mod.
        // We'll skip the heavy RA logic here, as is_in_test_module is already tested in other modules.
    }

    // ------------------------------------------------------------------------------
    //  test_derive_base_filename
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_derive_base_filename {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_function() {
            info!("Function => use the function name in snake_case");
            let c = dummy_ci_fn("AlphaBeta");
            let base = derive_base_filename(&c);
            assert_eq!(base, "alpha_beta");
        }

        #[traced_test]
        fn test_struct() {
            info!("Struct => use the struct name in snake_case");
            let c = dummy_ci_struct("SomeStruct");
            let base = derive_base_filename(&c);
            assert_eq!(base, "some_struct");
        }

        #[traced_test]
        fn test_empty_name() {
            info!("If item name is empty => 'untitled_item'");
            let c = dummy_ci_struct("");
            let base = derive_base_filename(&c);
            assert_eq!(base, "untitled_item");
        }

        #[traced_test]
        fn test_impl_for_struct() {
            info!("impl SomeTrait for Foo => produce 'impl_some_trait_for_foo'");
            let c = dummy_ci_impl("impl SomeTrait for Foo {}");
            let base = derive_base_filename(&c);
            assert_eq!(base, "impl_some_trait_for_foo");
        }

        #[traced_test]
        fn test_inherent_impl() {
            info!("impl SomeType => 'impl_for_sometype'");
            let c = dummy_ci_impl("impl SomeType");
            let base = derive_base_filename(&c);
            assert_eq!(base, "impl_for_sometype");
        }
    }

    // ------------------------------------------------------------------------------
    //  test_parse_impl_filename_fragment
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_parse_impl_filename_fragment {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_trait_for_type() {
            info!("trait => 'impl_some_trait_for_thing'");
            let s = parse_impl_filename_fragment("impl SomeTrait for Thing").unwrap();
            assert_eq!(s, "impl_some_trait_for_thing");
        }

        #[traced_test]
        fn test_inherent() {
            info!("inherent => 'impl_for_foo'");
            let s = parse_impl_filename_fragment("impl Foo").unwrap();
            assert_eq!(s, "impl_for_foo");
        }

        #[traced_test]
        fn test_generics() {
            info!("Ignore generics => 'impl_some_trait_for_bar'");
            let x = parse_impl_filename_fragment("impl SomeTrait<T> for Bar<U> {}").unwrap();
            assert_eq!(x, "impl_some_trait_for_bar");
        }

        #[traced_test]
        fn test_empty() {
            info!("Empty after 'impl' => None");
            let x = parse_impl_filename_fragment("impl   ");
            assert!(x.is_none());
        }
    }

    // ------------------------------------------------------------------------------
    //  test_make_unique_filename
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_make_unique_filename {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_first_use() {
            info!("No existing usage => 'apple.rs'");
            let mut used = HashSet::new();
            let p = make_unique_filename("apple", &mut used);
            assert_eq!(p, PathBuf::from("apple.rs"));
            assert!(used.contains("apple.rs"));
        }

        #[traced_test]
        fn test_collision() {
            info!("If 'apple.rs' is used, next => 'apple_2.rs', etc.");
            let mut used = HashSet::new();
            used.insert("apple.rs".to_string());
            let p = make_unique_filename("apple", &mut used);
            assert_eq!(p, PathBuf::from("apple_2.rs"));
            assert!(used.contains("apple_2.rs"));
            let p2 = make_unique_filename("apple", &mut used);
            // now apple.rs & apple_2.rs are used => next is apple_3.rs
            assert_eq!(p2, PathBuf::from("apple_3.rs"));
        }
    }

    // ------------------------------------------------------------------------------
    //  test_append_item_snippet
    // ------------------------------------------------------------------------------
    #[cfg(test)]
    mod test_append_item_snippet {
        use super::*;
        use tracing_test::traced_test;

        #[traced_test]
        fn test_empty_file_text() {
            info!("Start with empty => appended snippet plus newline");
            let mut file_text = String::new();
            append_item_snippet(&mut file_text, "fn something(){}");
            assert!(file_text.starts_with('\n'), "Should have a leading newline");
            assert!(file_text.ends_with('\n'), "Should have trailing newline");
            assert!(file_text.contains("fn something(){}"));
        }

        #[traced_test]
        fn test_already_ends_with_newline() {
            info!("If we already end with newline => just append snippet, ensure final newline");
            let mut file_text = "some existing text\n".to_string();
            append_item_snippet(&mut file_text, "struct Demo {}");
            // check intermediate newline
            assert!(file_text.contains("some existing text\nstruct Demo {}"), "No double blank line in between");
            assert!(file_text.ends_with('\n'), "Should end with newline");
        }

        #[traced_test]
        fn test_no_newline_in_snippet() {
            info!("Snippet has no trailing newline => we add it");
            let mut file_text = String::new();
            append_item_snippet(&mut file_text, "blah blah");
            assert!(file_text.ends_with('\n'), "We add trailing newline");
        }
    }
}
