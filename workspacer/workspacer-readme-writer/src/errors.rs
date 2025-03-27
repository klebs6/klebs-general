// ---------------- [ File: workspacer-readme-writer/src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum AiReadmeWriterError {
        LanguageModelBatchWorkflowError(LanguageModelBatchWorkflowError),
        BatchWorkspaceError(BatchWorkspaceError),
        WorkspacerFallbackError(WorkspacerFallbackError),
        CrateError(CrateError),
        WorkspaceError(WorkspaceError),
        CargoTomlError(CargoTomlError),
        #[display("ReadmeWriteError: {0}")]
        ReadmeWriteError(ReadmeWriteError),
    }
}
