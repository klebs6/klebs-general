crate::ix!();

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum ErrorReason {
    Io,
    Config,
    Parse,
    MissingData,
    Permission,
    InvalidArg,
    Unknown,
}

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum AppError {
    Io {
        code: std::io::ErrorKind,
    },
    Config {
        reason: ErrorReason,
    },
    Parse {
        reason: ErrorReason,
    },
    InvalidInput {
        reason: ErrorReason,
    },
    Generic {
        reason: ErrorReason,
    },
}

pub type AppResult<T> = Result<T, AppError>;
