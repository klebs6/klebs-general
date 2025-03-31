// ---------------- [ File: ai-json-template-derive/tests/trybuild/fail_unnamed_struct.rs ]
// ======================= File: tests/trybuild/fail_unnamed_struct.rs =======================
// A tuple/unnamed struct. The derive macro should fail with an error
// because AiJsonTemplate only supports named structs.

#![allow(dead_code)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_derive::*;
use save_load_traits::*;

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct FailUnnamedStruct(String, i32);

fn main() {}
