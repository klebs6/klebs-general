// ---------------- [ File: src/lib.rs ]
#![allow(unused_imports)]

#[inline]
pub fn is_test_mode() -> bool {
    // This variable is set automatically by Cargo when running tests.
    // cfg!(test) doesn’t always work for integration tests in separate crates,
    // so we do an environment var check:
    std::env::var("CARGO_RUNNING_TEST").is_ok()
}

pub use structopt::{self,StructOpt};
pub use tokio::runtime::Runtime as TokioRuntime;
pub use std::borrow::Cow;
pub use strum::{self,VariantNames,IntoEnumIterator};
pub use strum_macros::{Display as StrumDisplay, EnumIter, EnumString, EnumVariantNames};
pub use once_cell::unsync::OnceCell;
pub use async_openai::{
    self,
    types::{
        OpenAIFile,
        Batch, 
        BatchCompletionWindow, 
        BatchEndpoint, 
        BatchRequest, 
        BatchRequestInput, 
        BatchRequestInputMethod, 
        BatchRequestOutput,
        BatchStatus,
        CreateFileRequest, 
        FilePurpose,
    },
    Client,
    config::OpenAIConfig,
    error::{OpenAIError, ApiError as OpenAIApiError},
};
pub use serde::{de,de::DeserializeOwned,Deserializer,Deserialize,Serialize,Serializer};
pub use bytes::Bytes;

pub use export_magic::*;

pub use error_tree::*;
pub use std::path::{PathBuf,Path};
pub use std::fmt::{Formatter,Display,Debug,Result as FmtResult};
pub use std::str::FromStr;
pub use tokio::sync::mpsc::{channel,error::TrySendError,Sender,Receiver};
pub use tokio::{
    task::JoinSet,
    fs::{self,File},
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    self,
    sync::{
        RwLock as AsyncRwLock,
        RwLockReadGuard as AsyncRwLockReadGuard,
        RwLockWriteGuard as AsyncRwLockWriteGuard,
        Mutex as AsyncMutex,mpsc, oneshot, Semaphore,OwnedSemaphorePermit},
    task::JoinHandle,
    time::{Instant,sleep,Duration},
};
pub use indoc::{formatdoc,indoc};
pub use std::iter::Enumerate;
pub use std::slice::Chunks;
pub use inflector::cases::snakecase::to_snake_case;
pub use delegate::delegate;
pub use regex::{self,Regex,Captures};
pub use std::collections::{VecDeque,HashSet};
pub use traced_test::traced_test;
pub use tracing_setup::*;
pub use tracing::{self,instrument,trace,info,debug,error,warn};
pub use std::cell::RefCell;
pub use std::rc::Rc;
pub use tempfile::{self,tempdir,TempDir};
pub use std::cmp::Ordering;
pub use uuid::{self,Uuid};
pub use std::sync::{Arc};
pub use std::ops::Deref;

pub use futures::{self,future::join_all,executor::block_on};
pub use serde_json::Value as JsonValue;
pub use std::{
    collections::HashMap,
    io::{BufRead, BufWriter, Write},
};

pub use std::io::ErrorKind;
pub use async_openai::types::{
    BatchRequestOutputResponse,
    ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestUserMessageContent,
    ImageDetail,
    ImageUrl,
};

pub use export_magic::*;

pub use pbx::arc;
pub use std::error::Error;
pub use disable_macro::*;
pub use async_trait::async_trait;
pub use serde_json::{self,Value,json};
pub use strsim;
pub use std::str::Chars;
pub use std::iter::Peekable;
pub use json_repair::*;
pub use getset::*;
pub use derive_builder::{Builder};
pub use rand;

pub use ai_descriptor::*;
pub use rand_construct::*;
pub use structured_language_form::*;
pub use named_item::*;
pub use named_item_derive::NamedItem;
pub use std::thread::{self,ScopedJoinHandle,Scope};
pub use std::sync::atomic::{AtomicI32,AtomicUsize, Ordering as AtomicOrdering};
pub use crossbeam;
pub use std::marker::PhantomData;
pub use lazy_static::*;
pub use std::sync::Mutex as StdMutex;
pub use std::panic::{AssertUnwindSafe,catch_unwind};
pub use float_ord::FloatOrd;
pub use num_traits::Zero;
pub use std::pin::Pin;
