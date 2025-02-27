// ---------------- [ File: hydro2-3p/src/lib.rs ]
#![allow(unused_imports)]

pub use rand;
pub use pbx::arc;
pub use serde::{de,Deserializer,Deserialize,Serialize,Serializer};
pub use serde_json::Value as JsonValue;
pub use serde_json::{self,Value,json};
pub use lazy_static::*;
pub use traced_test::*;
pub use num_traits::Zero;
pub use async_trait::async_trait;
pub use std::borrow::Cow;
pub use std::cell::RefCell;
pub use std::cmp::Ordering;
pub use std::collections::{VecDeque,HashSet};
pub use std::error::Error;
pub use std::fmt::{Formatter,Display,Debug,Result as FmtResult};
pub use std::iter::Enumerate;
pub use std::iter::Peekable;
pub use std::marker::PhantomData;
pub use std::ops::Deref;
pub use std::panic::{AssertUnwindSafe,catch_unwind};
pub use std::path::{PathBuf,Path};
pub use std::rc::Rc;
pub use std::slice::Chunks;
pub use std::str::Chars;
pub use std::str::FromStr;
pub use std::sync::Mutex as StdMutex;
pub use std::sync::atomic::{AtomicI32,AtomicUsize, Ordering as AtomicOrdering};
pub use std::sync::{Arc};
pub use std::thread::{self,ScopedJoinHandle,Scope};
pub use std::{collections::HashMap, io::{BufRead, BufWriter, Write}, };
pub use export_magic::*;
pub use tokio::runtime::Runtime as TokioRuntime;
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
pub use futures::{self,future::join_all,executor::block_on};
pub use error_tree::*;
pub use disable_macro::*;
pub use tracing_setup::*;
pub use tracing::{info,debug,error,warn};
pub use named_item::*;
pub use named_item_derive::NamedItem;
pub use getset::*;
pub use derive_builder::{Builder};

/*
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
pub use bytes::Bytes;

pub use gpt_batch_scribe::{self,*};
//pub use gpt_batch_executor::*;

pub use indoc::{formatdoc,indoc};
pub use inflector::cases::snakecase::to_snake_case;
pub use delegate::delegate;
pub use regex::{self,Regex,Captures};
pub use tempfile::{self,tempdir,TempDir};
pub use uuid::{self,Uuid};


pub use async_openai::types::{
    BatchRequestOutputResponse,
    ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestUserMessageContent,
    ImageDetail,
    ImageUrl,
};

pub use export_magic::*;

pub use strsim;
pub use json_repair::*;

pub use ai_descriptor::*;
pub use rand_construct::*;
pub use structured_language_form::*;
pub use crossbeam;
pub use float_ord::FloatOrd;
*/
