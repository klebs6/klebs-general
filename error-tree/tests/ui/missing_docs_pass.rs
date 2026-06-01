#![deny(missing_docs)]

//! Compile-pass test for documented error_tree public output.

use error_tree::error_tree;

/// Payload used by the generated error type.
#[derive(Debug)]
pub struct Payload;

/// Inner payload used by the generated error type.
#[derive(Debug)]
pub struct InnerPayload;

error_tree! {
    /// Root error type must keep this documentation.
    pub enum RootError {
        /// Wrapped variant must keep this documentation.
        Wrapped(InnerError),

        /// Struct variant must keep this documentation.
        Structured {
            /// Struct field must keep this documentation.
            payload: Payload
        },
    }

    /// Inner error type must keep this documentation.
    pub enum InnerError {
        /// Inner wrapped payload must keep this documentation.
        Payload(InnerPayload),
    }
}

fn main() {}
