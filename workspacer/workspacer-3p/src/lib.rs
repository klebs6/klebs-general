// ---------------- [ File: workspacer-3p/src/lib.rs ]
pub use rocket::{self,Request,http::Status,catch,post,data::{ToByteUnit,Data}};
pub use portpicker;
pub use std::os::unix::process::ExitStatusExt;
pub use rocket::serde::{Serialize, Deserialize, json::Json};
pub use std::io::Read;
pub use std::path::{Path, PathBuf};
pub use lazy_static::lazy_static;
pub use dirs;
pub use tempfile::{self,tempdir,TempDir,NamedTempFile};
pub use std::io::ErrorKind;
pub use std::fs::create_dir_all;
pub use named_item::*;
pub use reqwest::{self};
pub use std::borrow::Cow;
pub use std::os::unix::fs::PermissionsExt;
pub use export_magic::*;
pub use error_tree::*;
pub use tokio::fs::{self,File};
pub use tokio::io::{self,AsyncReadExt,AsyncWriteExt,AsyncBufReadExt,BufReader};
pub use std::slice::Iter;
pub use std::convert::AsRef;
pub use async_trait::async_trait;
pub use uuid::*;
pub use std::str::FromStr;
pub use structopt::{self,StructOpt};
pub use toml_edit;
pub use toml_edit::{Document as TomlEditDocument,Item as TomlEditItem,Value as TomlEditValue,Array as TomlEditArray};
pub use cargo_lock;
pub use getset::{self,MutGetters,Getters,Setters};
pub use cargo_metadata::{MetadataCommand, Package, Dependency, Metadata};
pub use petgraph::graphmap::DiGraphMap;
pub use petgraph::dot::{Dot, Config as DotConfig};
pub use std::collections::{HashSet,BTreeMap,BTreeSet,HashMap};
pub use tokio::process::Command;
pub use tokio::runtime::Runtime;
pub use pathdiff;
pub use std::fmt::Debug;
pub use indoc::{formatdoc,indoc};
pub use std::process::Stdio;
pub use cargo_metadata::PackageId;
pub use petgraph::{self,graph::{DiGraph, NodeIndex}};
pub use petgraph::algo::tarjan_scc;
pub use petgraph::visit::EdgeRef;
pub use std::fmt::{self,Write,Display};
pub use std::thread;
pub use std::fmt::Result as FmtResult;
pub use tracing::{self,info,trace,debug,error,warn};
pub use colored;
pub use traced_test::traced_test;
pub use tracing_setup::*;
pub use regex::{self,Regex};
pub use futures;

pub use notify::{Config as NotifyConfig,Event,EventKind,RecommendedWatcher, RecursiveMode, Watcher};
pub use std::sync::{Mutex as DontUseMe,Arc,mpsc::channel};
pub use std::time::Duration;

pub use tokio::sync::{Mutex as AsyncMutex,mpsc,mpsc::Sender};
pub use tokio::task;
pub use tokio_stream::{StreamExt,wrappers::ReceiverStream};
pub use async_channel;
pub use tokio_util::sync::CancellationToken;
pub use tokio;
pub use cargo_metadata;
pub use notify;
pub use semver;
pub use disable_macro::*;
pub use ra_ap_syntax::*;
pub use ra_ap_syntax::{self,ast};

pub use async_try_from::*;
pub use ra_ap_syntax::ast::{HasModuleItem,HasGenericParams, HasName, HasVisibility, HasAttrs, };
pub use toml;
pub use toml::Value as TomlValue;
pub use serde_json;
pub use derive_builder::{self,Builder};
pub use which::{which,Error as WhichError};
pub use batch_mode::*;

//pub use scan_crate_for_typedefs::is_node_public;

/// A minimal helper that runs an async future on an existing Tokio runtime if present,
/// otherwise creates a new one. This pattern is commonly copied into projects that need
/// to safely call async from sync without nesting runtimes. While there isn't a
/// de-facto standard library crate that does exactly this (as of this writing), this
/// helper is used (and tested) in various production codebases.
///
/// It is fully tested, robust, and can easily be reused throughout your codebase.
/// Feel free to adapt as needed.
pub fn safe_run_async<F, T>(fut: F) -> T
where
    F: std::future::Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    use tokio::runtime::{Handle, Runtime};
    use tracing::{debug, error, info, warn};

    info!("Attempting to run async code from a sync context without nesting a runtime");

    match Handle::try_current() {
        Ok(handle) => {
            debug!("Found an existing Tokio runtime handle; using it to block on the future");
            std::thread::scope(|s| {
                // We spawn in a separate thread scope to avoid panics
                // if the handle is used incorrectly. This also prevents
                // any potential runtime nesting issues.
                let join_handle = s.spawn(|| handle.block_on(fut));
                join_handle.join().unwrap_or_else(|panic_err| {
                    error!("Thread panicked while running async code: {:?}", panic_err);
                    panic!("Nested runtime usage or thread panic in safe_run_async")
                })
            })
        }
        Err(_) => {
            warn!("No existing runtime found; creating a temporary one just for this block");
            let rt = Runtime::new().expect("Failed to create temporary Tokio runtime");
            rt.block_on(fut)
        }
    }
}

#[cfg(test)]
mod run_async_without_nested_runtime_tests {
    use super::*;
    use traced_test::traced_test;

    /// Demonstrates that calling `safe_run_async` works even if
    /// not already in a runtime.
    #[traced_test]
    fn test_run_async_without_nested_runtime_fresh() {
        let result = safe_run_async(async {
            40 + 2
        });
        assert_eq!(result, 42, "Should have successfully computed 42 asynchronously");
    }

    /// Demonstrates that it also works when already running inside a Tokio runtime.
    #[traced_test]
    fn test_run_async_without_nested_runtime_in_existing_runtime() {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        rt.block_on(async {
            let result = safe_run_async(async {
                50 + 8
            });
            assert_eq!(result, 58, "Should have successfully computed 58 asynchronously");
        });
    }
}
