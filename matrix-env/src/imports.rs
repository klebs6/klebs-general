// src/imports.rs

#![allow(unused_imports)]

pub(crate) use std::{
    ffi::OsString,
    fmt,
    future::Future,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    pin::Pin,
    process::ExitCode,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub(crate) use derive_builder::Builder;
pub(crate) use getset::Getters;
pub(crate) use serde::{Deserialize, Serialize};
pub(crate) use structopt::StructOpt;

pub(crate) use tokio::{
    runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
    sync::watch,
    task::{JoinError, JoinHandle},
    time,
};

pub(crate) use tracing::{debug, error, info, trace, warn, Instrument};

pub(crate) use tracing_subscriber::{
    filter::{EnvFilter, ParseError as EnvFilterParseError},
    fmt::{self as tracing_fmt, format::FmtSpan},
};

pub(crate) use export_magic::*;
pub(crate) use traced_test::*;
pub(crate) use tracing_setup::*;
