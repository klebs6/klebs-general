// ---------------- [ File: src/analyze_workspace.rs ]
crate::ix!();

#[async_trait]
impl Analyze for Workspace {

    type Analysis = WorkspaceSizeAnalysis;
    type Error    = WorkspaceError;

    async fn analyze(&self) -> Result<Self::Analysis, Self::Error> {

        let mut builder = WorkspaceSizeAnalysis::builder();

        for crate_handle in self {
            let crate_analysis = CrateAnalysis::new(crate_handle).await?;
            builder.add_crate_analysis(crate_analysis);
        }

        Ok(builder.build())
    }
}
