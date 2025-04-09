// ---------------- [ File: workspacer-consolidate/src/impl_block_interface.rs ]
crate::ix!();

#[derive(Serialize,Deserialize,Clone,Getters,Debug)]
#[getset(get="pub")]
pub struct ImplBlockInterface {
    docs:           Option<String>,
    attributes:     Option<String>,
    signature_text: String,
    methods:        Vec<CrateInterfaceItem<ast::Fn>>,
    type_aliases:   Vec<CrateInterfaceItem<ast::TypeAlias>>,

    /// The file from which this impl block was parsed
    file_path: PathBuf,

    /// The crate path that owns this impl block
    crate_path: PathBuf,

    /// The raw (untrimmed) range. Many tests expect to confirm it 
    /// matches the node’s actual text_range().
    raw_range: TextRange,

    /// The *trimmed* range, excluding leading/trailing normal comments 
    /// & whitespace. We'll use this in gather_interstitial_segments.
    effective_range: TextRange,
}

impl ImplBlockInterface {
    pub fn new_with_paths_and_range(
        docs:           Option<String>,
        attributes:     Option<String>,
        signature_text: String,
        methods:        Vec<CrateInterfaceItem<ast::Fn>>,
        type_aliases:   Vec<CrateInterfaceItem<ast::TypeAlias>>,
        file_path:      PathBuf,
        crate_path:     PathBuf,
        raw_range:      TextRange,
        effective_range: TextRange,
    ) -> Self {
        Self {
            docs,
            attributes,
            signature_text,
            methods,
            type_aliases,
            file_path,
            crate_path,
            raw_range,
            effective_range,
        }
    }

    #[cfg(test)]
    pub fn new_for_test(
        docs: Option<String>,
        attributes: Option<String>,
        signature_text: String,
        methods: Vec<CrateInterfaceItem<ast::Fn>>,
        type_aliases: Vec<CrateInterfaceItem<ast::TypeAlias>>,
    ) -> Self {
        Self::new_with_paths_and_range(
            docs,
            attributes,
            signature_text,
            methods,
            type_aliases,
            PathBuf::from("TEST_ONLY_file_path.rs"),
            PathBuf::from("TEST_ONLY_crate_path"),
            TextRange::new(0.into(), 0.into()),
            TextRange::new(0.into(), 0.into()),
        )
    }

    /// For interstitial logic, we want the *trimmed* range:
    pub fn text_range(&self) -> &TextRange {
        &self.effective_range
    }
}

impl fmt::Display for ImplBlockInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref docs) = self.docs {
            for line in docs.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        if let Some(ref attrs) = self.attributes {
            for line in attrs.lines() {
                writeln!(f, "{}", line)?;
            }
        }
        let sig = self.signature_text.trim_end();
        if self.methods.is_empty() && self.type_aliases.is_empty() {
            write!(f, "{} {{}}", sig)?;
            return Ok(());
        }
        writeln!(f, "{} {{", sig)?;

        for ta in &self.type_aliases {
            let txt = format!("{}", ta);
            for line in txt.lines() {
                writeln!(f, "    {}", line)?;
            }
        }
        for m in &self.methods {
            let txt = format!("{}", m);
            for line in txt.lines() {
                writeln!(f, "    {}", line)?;
            }
        }

        write!(f, "}}")?;
        Ok(())
    }
}

// Demonstrates testing the real `ImplBlockInterface` by parsing
// a snippet of Rust code and extracting the items via your actual code.
#[cfg(test)]
mod test_impl_block_interface_real {
    use super::*;
    // ^ Adjust imports to whatever your crate's structure is:
    //   - The real `ImplBlockInterface`
    //   - real `extract_docs`, `gather_all_attrs`
    //   - real `generate_impl_signature`
    //   - real `gather_impl_methods`, `gather_assoc_type_aliases`
    //   - real `ConsolidationOptions`

    /// Parses the snippet and returns the first `ast::Impl` node, if any.
    fn parse_first_impl(snippet: &str) -> Option<ast::Impl> {
        let parse = SourceFile::parse(snippet,Edition::Edition2024);
        let file_syntax = parse.tree().syntax().clone();
        for node in file_syntax.descendants() {
            if let Some(impl_node) = ast::Impl::cast(node) {
                return Some(impl_node);
            }
        }
        None
    }

    /// For demonstration, we define minimal versions of your real gather logic here.
    /// In real code, you might call `gather_impl_methods(&impl_ast, &options)` etc.
    fn gather_methods(impl_ast: &ast::Impl, options: &ConsolidationOptions)
        -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::Fn>>
    {
        let mut result = vec![];
        if let Some(assoc_items) = impl_ast.assoc_item_list() {
            for child in assoc_items.syntax().children() {
                if child.kind() == ra_ap_syntax::SyntaxKind::FN {
                    if let Some(fn_ast) = ast::Fn::cast(child.clone()) {
                        // Possibly skip private/test if you want:
                        if should_skip_item(&child, options) { 
                            continue; 
                        }
                        let docs = if *options.include_docs() {
                            extract_docs(&child)
                        } else {
                            None
                        };
                        let attrs = gather_all_attrs(&child);
                        // We don't set body_source in this example
                        let fn_item = CrateInterfaceItem::new_for_test(
                            fn_ast,
                            docs,
                            attrs,
                            None,
                            Some(options.clone())
                        );
                        result.push(fn_item);
                    }
                }
            }
        }
        result
    }

    fn gather_type_aliases(impl_ast: &ast::Impl, options: &ConsolidationOptions)
        -> Vec<crate::crate_interface_item::CrateInterfaceItem<ast::TypeAlias>>
    {
        let mut result = vec![];
        if let Some(assoc_items) = impl_ast.assoc_item_list() {
            for child in assoc_items.syntax().children() {
                if child.kind() == ra_ap_syntax::SyntaxKind::TYPE_ALIAS {
                    if let Some(ty_ast) = ast::TypeAlias::cast(child.clone()) {
                        if should_skip_item(&child, options) {
                            continue;
                        }
                        let docs = if *options.include_docs() {
                            extract_docs(&child)
                        } else {
                            None
                        };
                        let attrs = gather_all_attrs(&child);
                        let alias_item = CrateInterfaceItem::new_for_test(
                            ty_ast,
                            docs,
                            attrs,
                            None,
                            Some(options.clone())
                        );
                        result.push(alias_item);
                    }
                }
            }
        }
        result
    }

    /// Minimal stubs for skip logic, doc extraction, attribute gathering, etc.
    /// In actual code, you’d import the real versions.
    fn should_skip_item(_node: &SyntaxNode, _options: &ConsolidationOptions) -> bool {
        // For demonstration, we'll skip nothing
        false
    }

    fn extract_docs(_node: &SyntaxNode) -> Option<String> {
        // In real usage, you'd have a function that reads doc comments.
        // For demonstration, we rely on `#[doc="..."]` or `///...`.
        // Return None or Some(...) as you see fit.
        None
    }

    fn gather_all_attrs(_node: &SyntaxNode) -> Option<String> {
        // Real code might parse and convert them into lines. For demonstration:
        None
    }

    #[test]
    fn test_impl_block_real_code() {
        // A snippet with doc lines, attributes, a couple of methods, and a type alias
        let snippet = r#"
            /// This is doc line
            #[some_attr]
            impl MyTrait for MyType {
                fn do_stuff(&self) {}
                type AliasA = i32;
            }
        "#;

        let impl_ast = parse_first_impl(snippet).expect("Expected an impl");
        let mut options = ConsolidationOptions::new().with_docs();

        // Gather real docs & attributes from the impl node itself
        let docs = if *options.include_docs() {
            extract_docs(impl_ast.syntax())
        } else {
            None
        };
        let attrs = gather_all_attrs(impl_ast.syntax());

        // Generate the signature line: "impl MyTrait for MyType"
        let signature = generate_impl_signature(&impl_ast, docs.as_ref());

        // Gather methods & type aliases
        let methods = gather_methods(&impl_ast, &options);
        let aliases = gather_type_aliases(&impl_ast, &options);

        // Finally build the real ImplBlockInterface
        let ib = ImplBlockInterface::new_for_test(docs, attrs, signature, methods, aliases);

        // Format and compare with expected
        let output = format!("{}", ib);
        let expected = r#"impl MyTrait for MyType {
    type AliasA = i32;
    fn do_stuff(&self) {}
}"#;

        assert_eq!(output, expected);
    }

    #[test]
    fn test_impl_block_empty() {
        let snippet = r#"
            impl EmptyTrait for Unit {}
        "#;

        let impl_ast = parse_first_impl(snippet).expect("Expected an impl");
        let options = ConsolidationOptions::new(); // no docs

        let docs = extract_docs(impl_ast.syntax());
        let attrs = gather_all_attrs(impl_ast.syntax());
        let signature = generate_impl_signature(&impl_ast, docs.as_ref());

        let methods = gather_methods(&impl_ast, &options);
        let aliases = gather_type_aliases(&impl_ast, &options);

        let ib = ImplBlockInterface::new_for_test(docs, attrs, signature, methods, aliases);
        let output = format!("{}", ib);

        // No items => "impl EmptyTrait for Unit {}"
        assert_eq!(output, "impl EmptyTrait for Unit {}");
    }
}
