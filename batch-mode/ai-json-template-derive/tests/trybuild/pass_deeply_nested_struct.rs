#![allow(unused_imports)]
// ---------------- [ File: tests/trybuild/pass_deeply_nested_struct.rs ]
// ======================= File: tests/trybuild/pass_deeply_nested_struct.rs =======================
// Demonstrates multiple levels of nesting among structs, all deriving AiJsonTemplate.
// This should compile successfully.

#![allow(dead_code)]

use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};
use getset::{Getters, Setters};
use derive_builder::Builder;
use save_load_traits::*;
use save_load_derive::*;

/// Most deeply nested struct in the chain.
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct LevelThree {
    /// Info at level three
    l3_info: String,
}

/// Middle layer struct, nesting LevelThree
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct LevelTwo {
    /// Info at level two
    l2_info: String,

    /// Third level inside
    lvl_three: LevelThree,
}

/// Top-level struct with multiple nested layers
#[derive(SaveLoad,Clone, Debug, Serialize, Deserialize, Getters, Setters, Builder)]
#[derive(AiJsonTemplate)]
#[getset(get = "pub", set = "pub")]
#[builder(setter(into))]
struct LevelOne {
    /// Title at level one
    l1_title: String,

    /// Middle layer inside
    lvl_two: LevelTwo,
}

fn main() {}
