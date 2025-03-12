// ---------------- [ File: tests/trybuild/pass_nested_ok.rs ]
use ai_json_template_derive::*;
use ai_json_template::*;
use save_load_traits::*;
use batch_mode_3p::*;
use serde::{Serialize, Deserialize};

#[derive(Debug,Clone,AiJsonTemplate, Serialize, Deserialize)]
struct SubNested {
    sub_data: String,
}

impl_default_save_to_file_traits!{SubNested}

#[derive(Debug,Clone,AiJsonTemplate, Serialize, Deserialize)]
struct OkNested {
    main_text: String,
    sub: SubNested,
}

impl_default_save_to_file_traits!{OkNested}

fn main() {
    // This should compile successfully. 
    // The harness calls `t.pass("pass_nested_ok.rs")`.
    println!("All good!");
}
