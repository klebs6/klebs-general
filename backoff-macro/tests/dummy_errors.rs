// tests/dummy_errors.rs

use thiserror::Error;
use std::string::FromUtf8Error;
use std::io;

#[derive(Debug, Error)]
pub enum DummyError {
    #[error("default error")]
    Default,

    #[error("UTF-8 error")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("IO error")]
    IoError(#[from] io::Error),
    #[error("custom error")]
    Custom(String),
}

// Simulate more complex errors
#[derive(Debug, Error)]
pub enum ComplexError {
    #[error("Dummy error: {0}")]
    Dummy(#[from] DummyError),
    
    #[error("Other error")]
    Other,
}
