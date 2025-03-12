// ---------------- [ File: tests/trybuild/pass_nested_ok.rs ]
use ai_json_template_derive::*;
use ai_json_template::*;
use save_load_traits::*;
use save_load_derive::*;
use batch_mode_3p::*;
use serde::{Serialize, Deserialize};

#[derive(SaveLoad,Debug,Clone,AiJsonTemplate, Serialize, Deserialize)]
struct SubNested {
    sub_data: String,
}

#[derive(SaveLoad,Debug,Clone,AiJsonTemplate, Serialize, Deserialize)]
struct OkNested {
    main_text: String,
    sub: SubNested,
}

fn main() {
    // This should compile successfully. 
    // The harness calls `t.pass("pass_nested_ok.rs")`.
    println!("All good!");
}
