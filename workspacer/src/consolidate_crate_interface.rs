// ---------------- [ File: src/consolidate_crate_interface.rs ]
crate::ix!();

use tokio::fs;

#[async_trait]
impl ConsolidateCrateInterface for CrateHandle {
    async fn consolidate_crate_interface(&self) 
        -> Result<ConsolidatedCrateInterface, WorkspaceError> 
    {
        let source_files = self.source_files_excluding(&[]).await?;
        let mut interface = ConsolidatedCrateInterface::new();

        for file in source_files {
            // 1) read file contents
            let code = fs::read_to_string(&file).await.map_err(|e| {
                WorkspaceError::IoError { io_error: Arc::new(e) }
            })?;

            // 2) parse the code (not the path)
            //    If you want an edition, e.g. 2021, do:
            let syntax = SourceFile::parse(&code, Edition::Edition2024).syntax_node();

            println!("Parsing file: {}", file.display());

            // 3) Now do the usual scanning over descendants:
            for node in syntax.descendants() {
                if !is_node_public(&node) {
                    continue;
                }

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

