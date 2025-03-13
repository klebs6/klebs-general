// ---------------- [ File: tests/integration.rs ]
use save_load_derive::*;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput,Meta,parse_macro_input};
use tracing::{warn,info,error,trace,debug};
use derive_builder::*;
use getset::*;
use serde::{Serialize,Deserialize};
use traced_test::*;
use tracing_setup::*;
use save_load_traits::*;
use tempfile::tempdir;


#[derive(SaveLoad,Debug, Builder, Getters, Setters, Serialize, Deserialize, PartialEq)]
// We keep fields private, exposed only via getters/setters from getset.
struct TestData {
    #[getset(get, set)]
    #[builder(default = "\"Hello\".to_string()")]
    field_a: String,

    #[getset(get, set)]
    #[builder(default = "42")]
    field_b: i32,
}

#[traced_test]
fn verifies_saveload_derive_for_testdata() {
    debug!("Starting test of the `SaveLoad` derive macro's interface.");

    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime");
    rt.block_on(async {
        let instance = TestDataBuilder::default()
            .field_a("Hello from builder".to_string())
            .field_b(123)
            .build()
            .expect("Failed to build TestData");

        let tmp_dir = tempdir().expect("Failed to create temp directory");
        let file_path = tmp_dir.path().join("test_data.json");

        info!("Attempting to save `TestData` instance to file.");
        instance.save_to_file(&file_path)
            .await
            .expect("Save operation failed");

        info!("Attempting to load `TestData` instance back from file.");
        let loaded = TestData::load_from_file(&file_path)
            .await
            .expect("Load operation failed");

        assert_eq!(instance, loaded, "Loaded instance differs from the original.");
    });
}

#[traced_test]
fn verifies_error_handling_for_missing_file() {
    debug!("Starting test of missing file error path.");

    let rt = tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime");
    rt.block_on(async {
        let result = TestData::load_from_file("non_existent_file.json").await;
        match result {
            Err(SaveLoadError::IoError(_)) => {
                warn!("Got the expected IoError for a missing file path.")
            },
            other => panic!("Expected IoError, got {:?}", other),
        }
    });
}
