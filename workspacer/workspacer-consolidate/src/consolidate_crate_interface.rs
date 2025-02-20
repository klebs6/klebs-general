// ---------------- [ File: src/consolidate_crate_interface.rs ]
crate::ix!();

#[async_trait]
pub trait ConsolidateCrateInterface {
    async fn consolidate_crate_interface(
        &self,
        options: &ConsolidationOptions
    ) -> Result<ConsolidatedCrateInterface, CrateError>;
}

#[async_trait]
impl<T> ConsolidateCrateInterface for T
where
    T: CrateHandleInterface<PathBuf> + Sync + Send,
{
    async fn consolidate_crate_interface(
        &self,
        options: &ConsolidationOptions,
    ) -> Result<ConsolidatedCrateInterface, CrateError> {

        trace!("Consolidating crate interface.");

        let source_files = self.source_files_excluding(&[]).await?;
        let mut result = ConsolidatedCrateInterface::new();

        for file_path in source_files {
            let code         = self.read_file_string(&file_path).await?;
            let parse_result = SourceFile::parse(&code, Edition::Edition2024);
            let sf           = parse_result.tree();
            let items        = gather_crate_items(&sf, options);

            for item in items {
                match item {
                    ConsolidatedItem::Fn(ci)        => result.add_fn(ci),
                    ConsolidatedItem::Struct(ci)    => result.add_struct(ci),
                    ConsolidatedItem::Enum(ci)      => result.add_enum(ci),
                    ConsolidatedItem::Trait(ci)     => result.add_trait(ci),
                    ConsolidatedItem::TypeAlias(ci) => result.add_type_alias(ci),
                    ConsolidatedItem::Macro(ci)     => result.add_macro(ci),
                    ConsolidatedItem::Module(mi)    => result.add_module(mi),
                    ConsolidatedItem::ImplBlock(ib) => result.add_impl(ib),
                }
            }
        }

        info!("Crate interface consolidation complete.");
        Ok(result)
    }
}
