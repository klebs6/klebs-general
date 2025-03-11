// ---------------- [ File: tests/trybuild/pass_nested_ok.rs ]
use ai_json_template_derive::*;
use ai_json_template::*;
use serde::{Serialize, Deserialize};

#[derive(AiJsonTemplate, Serialize, Deserialize)]
struct SubNested {
    sub_data: String,
}

#[derive(AiJsonTemplate, Serialize, Deserialize)]
struct OkNested {
    main_text: String,
    sub: SubNested,
}

fn main() {
    // This should compile successfully. 
    // The harness calls `t.pass("pass_nested_ok.rs")`.
    println!("All good!");
}
