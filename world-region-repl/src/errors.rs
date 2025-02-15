// ---------------- [ File: src/errors.rs ]
crate::ix!();

error_tree!{

    pub enum PagerError {
        Default {
            msg: String,
        },
        TerminalSizeError,
        FailedToEnterAlternateScreen,
        ClearScreenError,
        MoveCursorError,
        WriteError,
        FlushError,
        PollError,
        ReadEventError,
        FailedToLeaveAltScreen,
        ReadlineError(rustyline::error::ReadlineError),
    }

    /// An error type if we cannot parse the user's partial input
    pub enum ValidateParseError {
        NoTokensProvided,
        NotAValidateCommand,
    }

    pub enum ValidateCompleteError {
        ValidateParseError(ValidateParseError),
    }

    pub enum ValidateCommandError {
        ValidateParseError(ValidateParseError),
    }
}
