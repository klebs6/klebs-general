// ---------------- [ File: workspacer-consolidate/src/consolidate_crate_interface.rs ]
crate::ix!();

#[async_trait]
pub trait ConsolidateCrateInterface {
    async fn consolidate_crate_interface(
        &self,
        options: &ConsolidationOptions
    ) -> Result<ConsolidatedCrateInterface, CrateError>;
}

// =========[ 2) consolidate_crate_interface(...) IMPL blocks fix ]==========
// File: workspacer-consolidate/src/consolidate_crate_interface.rs

#[async_trait]
impl<T> ConsolidateCrateInterface for T
where
    T: ReadFileString + GetSourceFilesWithExclusions + Sync + Send + RootDirPathBuf,
{
    async fn consolidate_crate_interface(
        &self,
        options: &ConsolidationOptions,
    ) -> Result<ConsolidatedCrateInterface, CrateError> {
        trace!("Consolidating crate interface.");

        let source_files = self.source_files_excluding(&[]).await?;
        let mut result = ConsolidatedCrateInterface::new();

        let crate_path = self.root_dir_path_buf();

        for file_path in source_files {
            let code = self.read_file_string(&file_path).await?;
            let parse_result = SourceFile::parse(&code, Edition::Edition2024);
            let sf = parse_result.tree();

            let items = gather_crate_items(&sf, options, &file_path, &crate_path);

            for item in items {
                match item {
                    ConsolidatedItem::Fn(ci0) => {
                        // we keep its raw/effective range
                        result.add_fn(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::Struct(ci0) => {
                        result.add_struct(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::Enum(ci0) => {
                        result.add_enum(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::Trait(ci0) => {
                        result.add_trait(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::TypeAlias(ci0) => {
                        result.add_type_alias(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::Macro(ci0) => {
                        result.add_macro(CrateInterfaceItem::new_with_paths_and_ranges(
                            ci0.item().as_ref().clone(),
                            ci0.docs().clone(),
                            ci0.attributes().clone(),
                            ci0.body_source().clone(),
                            ci0.consolidation_options().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *ci0.text_range(),
                            *ci0.effective_range(),
                        ));
                    }
                    ConsolidatedItem::ImplBlock(ib0) => {
                        // Now pass both raw_range and effective_range
                        result.add_impl(
                            ImplBlockInterface::new_with_paths_and_range(
                                ib0.docs().clone(),
                                ib0.attributes().clone(),
                                ib0.signature_text().clone(),
                                ib0.methods().clone(),
                                ib0.type_aliases().clone(),
                                file_path.clone(),
                                crate_path.clone(),
                                *ib0.raw_range(),      // 8th arg
                                *ib0.text_range(),     // 9th arg (the effective range)
                            )
                        );
                    }
                    ConsolidatedItem::Module(mi0) => {
                        // Similarly for module, we pass raw and effective:
                        let mut new_mod = ModuleInterface::new_with_paths_and_range(
                            mi0.docs().clone(),
                            mi0.attrs().clone(),
                            mi0.mod_name().clone(),
                            file_path.clone(),
                            crate_path.clone(),
                            *mi0.raw_range(),       // 6th arg
                            *mi0.text_range(),      // 7th arg (the effective range)
                        );
                        for sub_item in mi0.items().iter().cloned() {
                            new_mod.add_item(sub_item);
                        }
                        result.add_module(new_mod);
                    }
                    ConsolidatedItem::MockTest(_) => {
                        // skip or handle test mock variant
                    }
                }
            }
        }

        info!("Crate interface consolidation complete.");
        Ok(result)
    }
}

#[cfg(test)]
mod test_consolidate_crate_interface {
    use super::*;
    use std::path::{Path, PathBuf};
    use workspacer_3p::{tokio, async_trait}; // or your real imports
    use tempfile::tempdir;

    /// A minimal mock crate handle that stores an in-memory mapping of "file paths" to "file contents",
    /// and returns them for `source_files_excluding` + `read_file_string`.
    #[derive(Debug)]
    struct MockCrateHandle {
        // We'll store a vector of (PathBuf, file_content). We won't filter by excludes in this example,
        // but you could if needed.
        files: Vec<(PathBuf, String)>,
    }

    impl RootDirPathBuf for MockCrateHandle {
        fn root_dir_path_buf(&self) -> PathBuf {
            // Return whatever dummy path you like:
            PathBuf::from("TEST_ONLY_crate_root")
        }
    }


    impl MockCrateHandle {
        fn new() -> Self {
            Self { files: Vec::new() }
        }

        /// Add a "file" by specifying its path and content.
        fn add_file(&mut self, path: &str, content: &str) {
            self.files.push((PathBuf::from(path), content.to_string()));
        }
    }

    // We must implement `CrateHandleInterface<PathBuf>` in order to call
    // `consolidate_crate_interface(...)`, which requires T: CrateHandleInterface<PathBuf>.
    #[async_trait]
    impl GetSourceFilesWithExclusions for MockCrateHandle {
        // We only need to implement the methods that `consolidate_crate_interface` calls:
        //   - source_files_excluding(&[]) -> ...
        //   - read_file_string(...) -> ...
        // The rest can be stubs or unimplemented if not used.

        async fn source_files_excluding(
            &self,
            _exclude_files: &[&str],
        ) -> Result<Vec<PathBuf>, CrateError> {
            // Return all file paths from self.files. In real logic, you might filter out excludes.
            Ok(self.files.iter().map(|(p, _)| p.clone()).collect())
        }
    }

    #[async_trait]
    impl ReadFileString for MockCrateHandle {

        async fn read_file_string(&self, path: &Path) -> Result<String, CrateError> {
            // Return the snippet if we find a matching path
            if let Some((_, content)) = self.files.iter().find(|(p, _)| p == path) {
                Ok(content.clone())
            } else {
                Err(CrateError::FileNotFound {
                    missing_file: path.to_path_buf(),
                })
            }
        }
    }

    // Also need to implement `AsyncTryFrom<P>` if your code calls `CrateHandle::new(&P)` for a T
    // but if your code never calls that for the mock, you can skip. We'll stub:
    #[async_trait]
    impl AsyncTryFrom<PathBuf> for MockCrateHandle {
        type Error = CrateError;
        async fn new(_crate_path: &PathBuf) -> Result<Self, Self::Error> {
            unimplemented!()
        }
    }

    // ------------------------------------------------------------------------
    // Test `consolidate_crate_interface(...)` with multiple scenarios
    // ------------------------------------------------------------------------

    #[tokio::test]
    async fn test_empty_crate_no_source_files() {
        let handle = MockCrateHandle::new();
        let opts = ConsolidationOptions::new(); // all toggles off
        let cci = handle.consolidate_crate_interface(&opts).await.expect("Should succeed even if empty");
        // Expect an empty ConsolidatedCrateInterface
        assert!(cci.fns().is_empty(), "No files => no fns");
        assert!(cci.structs().is_empty(), "No structs");
        // etc. We can check all categories are empty
        assert!(cci.enums().is_empty());
        assert!(cci.traits().is_empty());
        assert!(cci.type_aliases().is_empty());
        assert!(cci.macros().is_empty());
        assert!(cci.impls().is_empty());
        assert!(cci.modules().is_empty());
    }

    /// 2) Single .rs file containing a basic function => ensure we gather a single ConsolidatedItem::Fn
    #[tokio::test]
    async fn test_single_fn_in_one_file() {
        let mut handle = MockCrateHandle::new();
        // We'll add a file named "main.rs" with a snippet that has exactly one fn
        let snippet = r#"
            fn foo() {}
        "#;
        handle.add_file("main.rs", snippet);

        let opts = ConsolidationOptions::new().with_private_items().with_fn_bodies().with_docs();
        let cci = handle.consolidate_crate_interface(&opts).await.expect("Should parse one fn");

        assert_eq!(cci.fns().len(), 1, "We should find exactly one function");
        assert!(cci.structs().is_empty(), "No structs expected");
        // etc...
        // If you want, you can check the cci.fns()[0].docs(), body, or signature if gather_crate_items is robust
    }

    /// 3) If the user code has multiple files, each with multiple items, we confirm they're all aggregated.
    ///    We'll do a snippet with a struct in file1, an enum + fn in file2, etc. 
    #[tokio::test]
    async fn test_multiple_files_with_multiple_items() {
        let mut handle = MockCrateHandle::new();

        let file1 = r#"
            struct Apple;
            fn eat_apple() {}
        "#;
        handle.add_file("file1.rs", file1);

        let file2 = r#"
            enum Fruit { Banana, Mango }
            trait Edible { fn eat(&self); }
        "#;
        handle.add_file("file2.rs", file2);

        let opts = ConsolidationOptions::new().with_docs().with_private_items(); // no fn bodies
        let cci = handle.consolidate_crate_interface(&opts).await.expect("Should parse multiple items");
        // We expect 1 struct (Apple), 1 fn (eat_apple), 1 enum (Fruit), 1 trait (Edible)
        assert_eq!(cci.structs().len(), 1, "We have 1 struct");
        assert_eq!(cci.fns().len(), 1, "We have 1 fn");
        assert_eq!(cci.enums().len(), 1, "We have 1 enum");
        assert_eq!(cci.traits().len(), 1, "We have 1 trait");
    }

    /// 4) If a file has test items (e.g. `#[cfg(test)] fn test_stuff()`) and we haven't enabled `with_test_items()`,
    ///    they should be skipped by `gather_crate_items`.
    #[tokio::test]
    async fn test_skips_test_items_without_with_test_items() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            #[cfg(test)]
            fn test_thing() {}

            fn normal_thing() {}
        "#;
        handle.add_file("file.rs", snippet);

        let opts = ConsolidationOptions::new().with_private_items(); // no .with_test_items()
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();
        // We expect only normal_thing
        assert_eq!(cci.fns().len(), 1, "Only normal_thing, skipping test_thing");
    }

    /// 5) If we call with_test_items(), then the test items are included. 
    #[tokio::test]
    async fn test_includes_test_items_with_option() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            #[cfg(test)]
            fn test_thing() {}

            fn normal_thing() {}
        "#;
        handle.add_file("file.rs", snippet);

        let opts = ConsolidationOptions::new().with_private_items().with_test_items();
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();
        // We expect 2 fns
        assert_eq!(cci.fns().len(), 2, "We include test_thing now");
    }

    /// 6) If we do `with_only_test_items()`, we skip normal items and only keep test items. 
    #[tokio::test]
    async fn test_only_test_items_skips_normal_items() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            #[cfg(test)]
            fn test_only() {}

            fn normal() {}
        "#;
        handle.add_file("file.rs", snippet);

        let opts = ConsolidationOptions::new().with_only_test_items();
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();
        // Now only test items => test_only is included, normal is not
        assert_eq!(cci.fns().len(), 1, "We have only test_only");
        let display = format!("{}", cci);
        // If you want, check the name or the snippet for the single fn
        // ...
    }

    /// 7) If we define private items (e.g. `struct PrivateItem;` without `pub`), we skip them if `with_private_items()` is not set.
    ///    We'll do a snippet with a private struct, a public struct, and see that only the public one is included if we skip private.
    #[tokio::test]
    async fn test_skips_private_items_by_default() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            struct PrivateThing;
            pub struct PublicThing;
        "#;
        handle.add_file("file.rs", snippet);

        let opts = ConsolidationOptions::new(); // no with_private_items
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();
        // We might expect gather_crate_items to skip private. 
        // This depends on your `should_skip_item` logic. 
        // We'll assume it checks if there's a `pub` token for struct or something.
        // So we get 1 struct: "PublicThing"
        assert_eq!(cci.structs().len(), 1, "One public struct only");

        // Now if we turn on with_private_items
        let opts2 = ConsolidationOptions::new().with_private_items();
        let cci2 = handle.consolidate_crate_interface(&opts2).await.unwrap();
        assert_eq!(cci2.structs().len(), 2, "We include both private + public now");
    }

    /// 8) If the snippet has an `impl` block or modules, we confirm they appear in `impls` or `modules` of the result.
    #[tokio::test]
    async fn test_impl_and_module_items() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            mod submod {
                fn inside_mod() {}
            }

            impl SomeTrait for Foo {
                fn trait_method(&self) {}
            }
        "#;
        handle.add_file("file.rs", snippet);

        let opts = ConsolidationOptions::new().with_private_items().with_test_items().with_docs(); 
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();

        // We expect 1 module and 1 impl block
        assert_eq!(cci.modules().len(), 1, "Should gather the submod");
        assert_eq!(cci.impls().len(), 1, "Should gather the impl block");
        // If the submod or impl block has internal items, they'd appear in the sub-lists of those structures 
        // or in child items if your logic recurses. 
        // We can do partial checks if needed.
    }

    /// 9) We can do a “stress test” with multiple files, toggles, doc lines, etc. 
    ///    In real usage, you'd parse actual code. We'll skip details for brevity.
    #[tokio::test]
    async fn test_large_scenario() {
        let mut handle = MockCrateHandle::new();
        let snippet1 = r#"
            // a bunch of items 
            pub fn alpha() {}
            fn beta() {}
            #[cfg(test)]
            fn gamma_test() {}
        "#;
        let snippet2 = r#"
            pub struct PubStruct;
            struct PrivStruct;
            #[cfg(test)]
            struct TestStruct;
        "#;
        let snippet3 = r#"
            enum E { A, B }
            macro_rules! mymacro { () => {} }
        "#;
        handle.add_file("first.rs", snippet1);
        handle.add_file("second.rs", snippet2);
        handle.add_file("third.rs", snippet3);

        let opts = ConsolidationOptions::new()
            .with_private_items()
            .with_test_items()
            .with_docs()
            .with_fn_bodies(); // some random toggles
        let cci = handle.consolidate_crate_interface(&opts).await.unwrap();

        // Now we can check cci.fns, cci.structs, cci.enums, cci.macros, etc. 
        // For instance:
        assert_eq!(cci.fns().len(), 3, "alpha, beta, gamma_test all included because private/test are on");
        assert_eq!(cci.structs().len(), 3, "PubStruct, PrivStruct, TestStruct included");
        assert_eq!(cci.enums().len(), 1);
        assert_eq!(cci.macros().len(), 1);
        // etc.
    }

    /// 10) If there's a parse error in one file (malformed Rust), we might either skip or fail, depending on your logic.
    ///     We'll confirm we get a real error if we want that behavior. 
    #[tokio::test]
    async fn test_malformed_rust_in_one_file() {
        let mut handle = MockCrateHandle::new();
        let snippet = r#"
            fn incomplete( 
        "#;
        handle.add_file("bad.rs", snippet);

        let opts = ConsolidationOptions::new();
        // If gather_crate_items or SourceFile::parse can handle partial parse, maybe it won't fail. 
        // If your code calls parse_result.errors() and fails if any, you might see an error. 
        // We'll assume your code tolerates partial parse or returns an error:
        let result = handle.consolidate_crate_interface(&opts).await;
        match result {
            Ok(cci) => {
                // Possibly a partial parse => maybe no items found
                assert!(cci.fns().is_empty(), "We found no complete fn? So empty.");
            }
            Err(e) => {
                // Or your code might produce CrateError for parse error
                error!("Got error: {:?}", e);
            }
        }
    }
}
