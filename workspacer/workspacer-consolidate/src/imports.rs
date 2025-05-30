// ---------------- [ File: workspacer-consolidate/src/imports.rs ]
pub(crate) use workspacer_3p::*;
pub(crate) use workspacer_crate_interface::*;
pub(crate) use workspacer_errors::*;
pub(crate) use workspacer_syntax::*;
pub(crate) use ::serde_derive::{Serialize,Deserialize};
pub(crate) use ::serde::{Deserializer,ser::SerializeStruct};
pub(crate) use ::serde::de::{Error as DeError, MapAccess, Visitor};
pub(crate) use std::marker::PhantomData;
