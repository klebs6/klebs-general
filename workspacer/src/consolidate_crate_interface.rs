crate::ix!();

#[async_trait]
impl ConsolidateCrateInterface for CrateHandle {
    async fn consolidate_crate_interface(&self) 
        -> Result<ConsolidatedCrateInterface, WorkspaceError> 
    {
        let source_files = self.source_files_excluding(&[]).await?;

        let mut interface = ConsolidatedCrateInterface::new();

        for file in source_files {
            let path_str = file.to_str().expect("PathBuf should be able to get to str");
            println!("Parsing file: {}", path_str);  // Debugging statement

            let parse = SourceFile::parse(path_str, Edition::Edition2024).syntax_node();

            for node in parse.descendants() {
                // Check if it's public and log
                if !is_node_public(&node) {
                    continue;
                }

                println!("Found public node: {:?}", node.kind());  // Debugging statement

                let docs = extract_docs(&node);

                match_ast! {
                    match node {
                        ast::Fn(it) => {
                            println!("Public function: {:?}", it.name());
                            interface.add_fn(CrateInterfaceItem::new(it, docs));
                        },
                        ast::Struct(it) => {
                            println!("Public struct: {:?}", it.name());
                            interface.add_struct(CrateInterfaceItem::new(it, docs));
                        },
                        ast::Enum(it) => {
                            println!("Public enum: {:?}", it.name());
                            interface.add_enum(CrateInterfaceItem::new(it, docs));
                        },
                        ast::Trait(it) => {
                            println!("Public trait: {:?}", it.name());
                            interface.add_trait(CrateInterfaceItem::new(it, docs));
                        },
                        ast::TypeAlias(it) => {
                            println!("Public type alias: {:?}", it.name());
                            interface.add_type(CrateInterfaceItem::new(it, docs));
                        },
                        ast::MacroRules(it) => {
                            println!("Public macro: {:?}", it.name());
                            interface.add_macro(CrateInterfaceItem::new(it, docs));
                        },
                        _ => {}
                    }
                }
            }
        }

        Ok(interface)
    }
}
