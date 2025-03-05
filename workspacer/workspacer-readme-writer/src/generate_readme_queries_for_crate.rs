// ---------------- [ File: workspacer-readme-writer/src/generate_readme_queries_for_crate.rs ]
crate::ix!();

/// Implements generation of AI queries and final README writing for `CrateHandle`.
///  - We gather the consolidated interface for the crate.
///  - We build a set of queries to the AI describing what we want it to produce or update.
///  - We call `send_readme_queries_to_ai` (or your real AI client) to get responses.
///  - We write or update the README with `write_updated_readme`.
#[async_trait]
impl GenerateReadmeQueries for CrateHandle {
    type Error = CrateError;

    #[tracing::instrument(level="trace", skip_all)]
    async fn generate_readme_queries(
        &self,
        interface: &ConsolidatedCrateInterface
    ) -> Result<Vec<AiReadmeQuery>, Self::Error> {

        trace!("CrateHandle::generate_readme_queries: building the queries for AI");
        let name = self.name();

        let instructions = indoc!{r#"
        We have a crate named '{name}'. We would like you to write a README.md for it with maximally helpful content.
        Please tailor your response for an apex consumer of maximal intelligence.
        A reader of this README.md should have a good sense of what the crate does and how to use it after reading what you write.
        Here is the interface for the crate:
        "#};

        let interface_str = format!("{}", interface);

        // For a single crate, we might produce exactly one query that includes the entire interface.
        let single_query = AiReadmeQueryBuilder::default()
            .query_text(interface_str)
            .instructions(instructions)
            .build()
            .unwrap();

        Ok(vec![single_query])
    }
}
