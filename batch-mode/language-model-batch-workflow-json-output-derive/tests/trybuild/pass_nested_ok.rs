// ---------------- [ File: tests/trybuild/pass_nested_ok.rs ]
use language_model_batch_workflow_json_output_derive::*;
use batch_mode_batch_workflow::*;
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
