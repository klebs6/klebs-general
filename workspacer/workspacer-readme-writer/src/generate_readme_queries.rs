// ---------------- [ File: workspacer-readme-writer/src/generate_readme_queries.rs ]
crate::ix!();

/// A trait for generating the structured queries we will send to the AI based on the
/// crate’s or workspace’s consolidated crate interface, along with any system-level
/// instructions for how we want the AI to write the README.
#[async_trait]
pub trait GenerateReadmeQueries {
    type Error;

    /// Given a consolidated interface, produce one or more queries to the AI. 
    /// The queries might contain the items extracted from code, plus instructions
    /// about how to generate or update the README.
    async fn generate_readme_queries(
        &self,
        interface: &ConsolidatedCrateInterface
    ) -> Result<Vec<AiReadmeQuery>, Self::Error>;
}
