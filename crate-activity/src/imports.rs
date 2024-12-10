pub(crate) use reqwest::Client;
pub(crate) use futures::{future,StreamExt, TryStreamExt};
pub(crate) use serde::{Serialize,Deserialize};
pub(crate) use std::collections::HashMap;
pub(crate) use error_tree::*;
pub(crate) use tokio::{
    fs::{self,File},
    io::{self,AsyncBufReadExt,BufReader}
};
pub(crate) use std::path::{PathBuf,Path};
pub(crate) use chrono::{NaiveDate,DateTime, Utc};
pub(crate) use export_magic::*;
pub(crate) use getset::{Getters, Setters};
pub(crate) use derive_builder::Builder;
pub(crate) use std::fmt;
pub(crate) use itertools::Itertools; // For the minmax method
