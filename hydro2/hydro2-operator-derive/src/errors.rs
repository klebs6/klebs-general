// ---------------- [ File: hydro2-operator-derive/src/errors.rs ]
//! errors.rs — Central definitions of all parse-time or generation-time errors.
crate::ix!();

/// The category of parse/generation error for the operator macro.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OperatorSpecErrorKind {
    MissingOperatorAttribute,
    NonStringKeyValue,
    InvalidKey,
    DuplicateExecuteKey,
    DuplicateOpcodeKey,
    MissingExecuteFn,
    MissingOpcode,
    InvalidInputOrder,
    InvalidOutputOrder,
    MoreThanFourInputs,
    MoreThanFourOutputs,
    CouldNotParseExecutePath,
    CouldNotParseOpcodePath,
    CouldNotParseInputType,
    CouldNotParseOutputType,
}

/// A robust error type carrying both a specific kind and a `Span` for reporting.
#[derive(Getters,Debug)]
#[getset(get = "pub")]
pub struct OperatorSpecError {
    kind: OperatorSpecErrorKind,
    span: Span,
}

impl OperatorSpecError {
    // ------------------------------------------------------------------------
    //  Common builder methods for each variant
    // ------------------------------------------------------------------------
    pub fn missing_operator_attribute(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::MissingOperatorAttribute,
            span: *span,
        }
    }
    pub fn non_string_key_value(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::NonStringKeyValue,
            span: *span,
        }
    }
    pub fn invalid_key(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::InvalidKey,
            span: *span,
        }
    }
    pub fn duplicate_execute_key(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::DuplicateExecuteKey,
            span: *span,
        }
    }
    pub fn duplicate_opcode_key(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::DuplicateOpcodeKey,
            span: *span,
        }
    }
    pub fn missing_execute_fn(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::MissingExecuteFn,
            span: *span,
        }
    }
    pub fn missing_opcode(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::MissingOpcode,
            span: *span,
        }
    }
    pub fn invalid_input_order(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::InvalidInputOrder,
            span: *span,
        }
    }
    pub fn invalid_output_order(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::InvalidOutputOrder,
            span: *span,
        }
    }
    pub fn more_than_four_inputs(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::MoreThanFourInputs,
            span: *span,
        }
    }
    pub fn more_than_four_outputs(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::MoreThanFourOutputs,
            span: *span,
        }
    }
    pub fn could_not_parse_execute_path(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::CouldNotParseExecutePath,
            span: *span,
        }
    }
    pub fn could_not_parse_opcode_path(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::CouldNotParseOpcodePath,
            span: *span,
        }
    }
    pub fn could_not_parse_input_type(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::CouldNotParseInputType,
            span: *span,
        }
    }
    pub fn could_not_parse_output_type(span: &Span) -> Self {
        Self {
            kind: OperatorSpecErrorKind::CouldNotParseOutputType,
            span: *span,
        }
    }

    /// Convert this error into `compile_error!` tokens for user feedback at compile time.
    pub fn to_compile_error(&self) -> TokenStream {
        let msg = match self.kind {
            OperatorSpecErrorKind::MissingOperatorAttribute => {
                "Operator attribute `#[operator(...)]` is required."
            }
            OperatorSpecErrorKind::NonStringKeyValue => {
                "Non-string key-value attribute in `#[operator(...)]`."
            }
            OperatorSpecErrorKind::InvalidKey => {
                "Unexpected key in `#[operator(...)]`. Use `execute`, `opcode`, `input0..3`, `output0..3`, etc."
            }
            OperatorSpecErrorKind::DuplicateExecuteKey => {
                "Execute key was provided more than once in `#[operator(...)]`."
            }
            OperatorSpecErrorKind::DuplicateOpcodeKey => {
                "Opcode key was provided more than once in `#[operator(...)]`."
            }
            OperatorSpecErrorKind::MissingExecuteFn => {
                "No `execute` function name specified in `#[operator(...)]`."
            }
            OperatorSpecErrorKind::MissingOpcode => {
                "No `opcode` specified in `#[operator(...)]`."
            }
            OperatorSpecErrorKind::InvalidInputOrder => {
                "Input keys must be in strictly increasing order: input0, input1, etc."
            }
            OperatorSpecErrorKind::InvalidOutputOrder => {
                "Output keys must be in strictly increasing order: output0, output1, etc."
            }
            OperatorSpecErrorKind::MoreThanFourInputs => {
                "At most four inputs allowed: input0..input3."
            }
            OperatorSpecErrorKind::MoreThanFourOutputs => {
                "At most four outputs allowed: output0..output3."
            }
            OperatorSpecErrorKind::CouldNotParseExecutePath => {
                "Could not parse `execute` string as a path."
            }
            OperatorSpecErrorKind::CouldNotParseOpcodePath => {
                "Could not parse `opcode` string as a path."
            }
            OperatorSpecErrorKind::CouldNotParseInputType => {
                "Could not parse `inputN` string as a Type."
            }
            OperatorSpecErrorKind::CouldNotParseOutputType => {
                "Could not parse `outputN` string as a Type."
            }
        };

        syn::Error::new(self.span, msg).to_compile_error()
    }
}

// Optional: If you want to implement `std::error::Error` and `Display`, you can do so:
impl std::fmt::Display for OperatorSpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}", self.kind, self.span)
    }
}

impl std::error::Error for OperatorSpecError {}
