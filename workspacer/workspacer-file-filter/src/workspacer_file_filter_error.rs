crate::ix!();

error_tree!{

    pub enum AiFileFilterError {

        #[display("IoError: {context} => {io_error}")]
        IoError {
            io_error: std::sync::Arc<std::io::Error>,
            context:  String,
        },

        #[display("An error occurred in the cargo toml layer: {0}")]
        CargoTomlError(CargoTomlError),

        #[display("Error from the language model workflow: {0}")]

        LanguageModelBatchWorkflowError(LanguageModelBatchWorkflowError),
        #[display("Generic fallback error in AiFileFilter")]
        GenericError,
    }
}
