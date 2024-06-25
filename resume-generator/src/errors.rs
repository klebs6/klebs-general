crate::ix!();

error_tree!{

    pub enum CliError {
        NoOutputFilenameProvided,
        NoOutputDirectoryProvided,
    }

    pub enum ResumeBuilderError {
        CouldNotParseDate,
        CliError(CliError),
        IoError(std::io::Error),
    }
}
