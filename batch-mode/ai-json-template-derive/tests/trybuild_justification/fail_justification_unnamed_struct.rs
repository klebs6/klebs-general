// ---------------- [ File: ai-json-template-derive/tests/trybuild_justification/fail_justification_unnamed_struct.rs ]
// tests/trybuild_justification/fail_justification_unnamed_struct.rs
//
// A tuple struct deriving AiJsonTemplateWithJustification => must fail
// because we only support named fields.

#![allow(dead_code)]
#![allow(unused_imports)]

use ai_json_template::*;
use ai_json_template_derive::*;
use serde::*;
use getset::*;
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;
use tracing::*;

#[derive(SaveLoad, Debug, Clone, Serialize, Deserialize, Builder)]
#[derive(AiJsonTemplate, AiJsonTemplateWithJustification)]
struct BasicUnnamed(u32, String);

fn main() {}
