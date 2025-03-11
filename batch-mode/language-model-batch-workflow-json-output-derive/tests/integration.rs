// ---------------- [ File: tests/integration.rs ]
use language_model_batch_workflow_json_output_derive::*; // so we can call `.to_template()`
use batch_mode_batch_workflow::*; // so we can call `.to_template()`
use batch_mode_3p::*; // so we can call `.to_template()`
use serde::{Serialize, Deserialize};

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct MySimpleConfig {
    /// doc for name
    pub name: String,
    /// doc for optional description
    pub description: Option<String>,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct MyNested {
    /// doc for nested text
    pub sub_text: String,
    /// doc for nested tags
    pub sub_tags: Vec<String>,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct Outer {
    /// doc for outer text
    pub outer_text: String,
    /// doc for the nested struct
    pub nested: MyNested,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct ThirdLevel {
    /// doc for third-level data
    pub data: String,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct SecondLevel {
    /// doc for second-level note
    pub note: String,

    /// doc for further nesting
    pub third: ThirdLevel,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct FirstLevel {
    /// doc for top-level message
    pub message: String,

    /// doc for second-level nesting
    pub second: SecondLevel,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
pub struct Complex {
    /// doc for title
    pub title: String,

    /// doc for multiple tags
    pub tags: Vec<String>,

    /// doc for optional remark
    pub remark: Option<String>,
}

#[derive(Getters,AiJsonTemplate, Debug, Clone, Serialize, Deserialize)]
#[getset(get="pub")]
/// doc comment for MyDocCommented
pub struct MyDocCommented {
    /// doc for alpha
    pub alpha: String,

    /// doc for beta
    pub beta: String,
}

#[traced_test]
fn test_simple_struct_template() {
    trace!("Starting test_simple_struct_template");
    let tmpl = MySimpleConfig::to_template();
    info!("Generated template for MySimpleConfig: {}", serde_json::to_string_pretty(&tmpl).unwrap());

    // Round-trip test
    let original = MySimpleConfig {
        name: "Test".into(),
        description: Some("Some info".into()),
    };
    let as_json = serde_json::to_string(&original).unwrap();
    let back: MySimpleConfig = serde_json::from_str(&as_json).unwrap();
    assert_eq!(back.name(), "Test");
    assert_eq!(back.description().as_deref(), Some("Some info"));
}

#[traced_test]
fn test_nested_struct_template() {
    trace!("Starting test_nested_struct_template");
    let tmpl = Outer::to_template();
    info!("Template (Outer): {}", serde_json::to_string_pretty(&tmpl).unwrap());

    let tmpl_str = serde_json::to_string(&tmpl).unwrap();
    assert!(tmpl_str.contains("\"nested_template\""),
        "Should contain 'nested_template' for the nested struct."
    );

    // Round-trip
    let original = Outer {
        outer_text: "hello".to_string(),
        nested: MyNested {
            sub_text: "sub here".to_string(),
            sub_tags: vec!["tag1".to_string(), "tag2".to_string()],
        },
    };
    let as_json = serde_json::to_string(&original).unwrap();
    let back: Outer = serde_json::from_str(&as_json).unwrap();

    assert_eq!(back.outer_text(), "hello");
    assert_eq!(back.nested().sub_text(), "sub here");
    assert_eq!(*back.nested().sub_tags(), vec!["tag1".to_string(), "tag2".to_string()]);
}

#[traced_test]
fn test_deeply_nested_struct_template() {
    trace!("Starting test_deeply_nested_struct_template");
    let tmpl = FirstLevel::to_template();
    info!("Template (FirstLevel): {}", serde_json::to_string_pretty(&tmpl).unwrap());

    let tmpl_str = serde_json::to_string_pretty(&tmpl).unwrap();
    assert!(tmpl_str.contains("nested_template"),
        "Should contain at least one 'nested_template' reference for multi-level nesting."
    );

    let parsed: serde_json::Value = serde_json::from_str(&tmpl_str).unwrap();
    let struct_docs = parsed["struct_docs"].as_str().unwrap_or("");
    assert!(struct_docs.contains("doc for top-level message"),
        "Should contain doc comment for FirstLevel."
    );

    // Round-trip
    let original = FirstLevel {
        message: "root msg".to_string(),
        second: SecondLevel {
            note: "note content".to_string(),
            third: ThirdLevel { data: "deepest info".to_string() },
        },
    };
    let as_json = serde_json::to_string(&original).unwrap();
    let back: FirstLevel = serde_json::from_str(&as_json).unwrap();
    assert_eq!(back.message(), "root msg");
    assert_eq!(back.second().note(), "note content");
    assert_eq!(back.second().third().data(), "deepest info");
}

#[traced_test]
fn test_struct_with_vec_strings_and_more() {
    trace!("Starting test_struct_with_vec_strings_and_more");
    let tmpl = Complex::to_template();
    info!("Template (Complex): {}", serde_json::to_string_pretty(&tmpl).unwrap());

    let parsed: serde_json::Value = serde_json::from_value(tmpl.clone()).unwrap();
    assert!(parsed["fields"]["title"]["type"] == "string");
    assert!(parsed["fields"]["tags"]["type"] == "array_of_strings");
    assert!(parsed["fields"]["remark"]["required"] == false);

    // Round-trip
    let original = Complex {
        title: "Title".to_string(),
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        remark: None,
    };
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: Complex = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.title(), "Title");
    assert_eq!(deserialized.tags().len(), 2);
    assert_eq!(*deserialized.remark(), None);
}

#[traced_test]
fn test_doc_comment_in_output() {
    trace!("Starting test_doc_comment_in_output");
    let tmpl = MyDocCommented::to_template();
    let tmpl_str = serde_json::to_string_pretty(&tmpl).unwrap();
    info!("Template (MyDocCommented): {tmpl_str}");

    assert!(tmpl_str.contains("doc comment for MyDocCommented"),
        "Should contain doc comment for MyDocCommented in struct_docs"
    );
    assert!(tmpl_str.contains("doc for alpha"));
    assert!(tmpl_str.contains("doc for beta"));
}
