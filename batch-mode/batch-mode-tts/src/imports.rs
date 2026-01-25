// ---------------- [ File: batch-mode-tts/src/imports.rs ]
// If we belong to a prefix group, we'd do `pub(crate) use prefix_3p::*;`
// For now, placeholder comment.
pub(crate) use batch_mode_3p::*;
//pub(crate) use export_magic::*;
//pub(crate) use tracing_setup::*;
pub(crate) use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) use derive_builder::Builder;
pub(crate) use getset::Getters;
pub(crate) use tracing::{debug, error, info, trace, warn};

pub(crate) use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateSpeechRequest, CreateSpeechRequestArgs, SpeechModel, SpeechResponseFormat, Voice,
    },
    Client as OpenAIClient,
};
