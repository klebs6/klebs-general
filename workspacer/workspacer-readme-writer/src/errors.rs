// ---------------- [ File: workspacer-readme-writer/src/errors.rs ]
crate::ix!();

error_tree!{
    pub enum ReadmeWriterExecutionError {
        LanguageModelBatchWorkflowError(LanguageModelBatchWorkflowError),
        WorkspacerFallbackError(WorkspacerFallbackError),
        CrateError(CrateError),
        CargoTomlError(CargoTomlError),
        #[display("ReadmeWriteError: {0}")]
        ReadmeWriteError(ReadmeWriteError),
    }
}
