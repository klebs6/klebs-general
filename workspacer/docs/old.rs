
pub struct ConsolidatedCrateInterface {
    traits:  Vec<ast::Trait>,
    fns:     Vec<ast::Fn>,
    structs: Vec<ast::Struct>,
    enums:   Vec<ast::Enum>,
    types:   Vec<ast::TypeAlias>,
    macros:  Vec<ast::MacroRules>,
}

#[async_trait]
impl ConsolidateCrateInterface for CrateHandle {

    async fn consolidate_crate_interface(&self) 
        -> Result<ConsolidatedCrateInterface, WorkspaceError> 
    {
        let source_files = self.source_files_excluding(&[]).await?;

        let mut traits  = vec![];
        let mut fns     = vec![];
        let mut structs = vec![];
        let mut enums   = vec![];
        let mut types   = vec![];
        let mut macros  = vec![];

        for file in source_files {

            let path_str = file.to_str().expect("PathBuf should be able to get to str");
            let parse    = SourceFile::parse(path_str, Edition::Edition2024).syntax_node();

            for node in parse.descendants() {

                if !is_node_public(&node) {
                    continue;
                }

                match_ast! {
                    match node {
                        ast::Fn(it)         => fns.push(it),
                        ast::Struct(it)     => structs.push(it),
                        ast::Enum(it)       => enums.push(it),
                        ast::Trait(it)      => traits.push(it),
                        ast::TypeAlias(it)  => types.push(it),
                        ast::MacroRules(it) => macros.push(it),
                        _ => { },
                    }
                }
            }
        }

        Ok(ConsolidatedCrateInterface {
            traits,
            fns,
            structs,
            enums,
            types,
            macros,
        })
    }
}
