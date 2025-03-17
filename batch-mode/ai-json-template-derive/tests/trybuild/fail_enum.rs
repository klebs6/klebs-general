// ---------------- [ File: tests/trybuild/fail_enum.rs ]
// ======================= File: tests/trybuild/fail_enum.rs =======================
// The macro should fail on an enum, since AiJsonTemplate only supports named structs.

#![allow(dead_code)]

use ai_json_template_derive::*;
use serde::{Serialize, Deserialize};
use save_load_derive::*;
use save_load_traits::*;

#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize)]
#[derive(AiJsonTemplate)]
enum FailEnum {
    VariantOne,
    VariantTwo,
}

fn main() {}
