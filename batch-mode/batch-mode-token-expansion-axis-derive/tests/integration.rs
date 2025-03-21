// ---------------- [ File: tests/integration.rs ]
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use batch_mode_3p::*;
use batch_mode_token_expansion_traits::*;
use batch_mode_token_expansion_axis_derive::*;
use save_load_traits::{LoadFromFile, SaveLoadError};
use save_load_derive::*;
use serde_json;

#[traced_test]
fn check_multiple_variants_and_expanded_struct() {
    info!("Defining an enum with multiple axis variants to test the expanded struct output.");

    #[derive(Debug, TokenExpansionAxis)]
    #[system_message_goal("Complex System Goal")]
    enum ComplexAxis {
        #[axis("Foo => Description for foo axis")]
        Foo,
        #[axis("Bar => Description for bar axis")]
        Bar,
        #[axis("Baz => Description for baz axis")]
        Baz,
    }

    // The derived aggregator is `ComplexAxisExpander`.
    // The derived expanded struct is `ExpandedComplexAxis`.
    let aggregator = ComplexAxisExpander::default();

    info!("Check aggregator name & system message goal.");
    assert_eq!(aggregator.name(), "ComplexAxisExpander");
    assert_eq!(aggregator.system_message_goal(), "Complex System Goal");

    debug!("Check aggregator axes.");
    let axes = aggregator.axes();
    assert_eq!(axes.len(), 3);

    info!("Check each axis_name / axis_description for correctness.");
    assert_eq!(axes[0].axis_name(), "Foo");
    assert_eq!(axes[0].axis_description(), "Description for foo axis");
    assert_eq!(axes[1].axis_name(), "Bar");
    assert_eq!(axes[1].axis_description(), "Description for bar axis");
    assert_eq!(axes[2].axis_name(), "Baz");
    assert_eq!(axes[2].axis_description(), "Description for baz axis");

    debug!("Check the derived data-carrying struct name. We can instantiate it and confirm it compiles.");
    let expanded = ExpandedComplexAxis {
        name: String::from("some_token"),
        foo: None,
        bar: Some(vec![String::from("sample")]),
        baz: None,
    };
    assert_eq!(expanded.name(), "some_token");
    assert_eq!(*expanded.bar(), Some(vec![String::from("sample")]));

    info!("check_multiple_variants_and_expanded_struct test complete.");
}

#[traced_test]
fn check_missing_axis_attribute_compile_fail() {
    info!("Defining an enum with a missing #[axis(...)] attribute for a variant should panic at compile time.");

    // Because this is a *compile-time* failure scenario, we cannot run it
    // as a normal run-time test. Instead, we rely on a UI test approach
    // or a doc-test style negative test. For demonstration here, we
    // simply illustrate that such code *would not compile*:
    //
    // #[derive(TokenExpansionAxis)]
    // enum BadAxis {
    //     NoAxisHere,
    // }
    //
    // We do not actually compile it, because that would cause a compile error
    // which is correct. So there's no run-time test to do here, but we log
    // the intent for completeness.
    info!("No run-time assert needed; this scenario is tested via compile-fail checks in a UI test harness.");
}

#[traced_test]
async fn check_load_from_file_in_expanded_struct() {
    info!("Verifying that the derived `ExpandedToken` struct implements `LoadFromFile` correctly.");

    #[derive(Debug, TokenExpansionAxis)]
    #[system_message_goal("Loadable System Goal")]
    enum LoadableAxis {
        #[axis("LOne => axis LOne")]
        LOne,
        #[axis("LTwo => axis LTwo")]
        LTwo,
    }

    // We expect an aggregator named `LoadableAxisExpander`,
    // and the expanded struct: `ExpandedLoadableAxis`.
    let aggregator = LoadableAxisExpander::default();
    let goal: Cow<'_, str> = aggregator.system_message_goal();
    assert_eq!(goal, "Loadable System Goal");

    info!("We'll create a temporary JSON file representing an `ExpandedLoadableAxis` object, then load it.");

    let original = ExpandedLoadableAxis {
        name: "MyLoadableToken".to_string(),
        l_one: Some(vec!["a".to_string(), "b".to_string()]),
        l_two: None,
    };

    trace!("Serialize it to JSON in a temp file.");
    let tmp_file_path: PathBuf = std::env::temp_dir().join("temp_load_test.json");
    fs::write(&tmp_file_path, serde_json::to_string_pretty(&original).unwrap())
        .expect("Unable to write temp JSON file");

    debug!("Now attempt to load it using the derived `LoadFromFile` impl.");
    let loaded = ExpandedLoadableAxis::load_from_file(&tmp_file_path)
        .await
        .expect("Failed to load from file");

    fs::remove_file(&tmp_file_path).ok();

    info!("Check that the loaded struct matches the original object.");
    assert_eq!(loaded.name(), "MyLoadableToken");
    assert_eq!(*loaded.l_one(), Some(vec!["a".to_string(), "b".to_string()]));
    assert_eq!(*loaded.l_two(), None);

    info!("check_load_from_file_in_expanded_struct test complete.");
}

#[traced_test]
fn check_missing_system_message_goal() {
    info!("Defining an enum without an explicit system_message_goal attribute.");

    #[derive(Debug, TokenExpansionAxis)]
    enum SimpleAxis {
        #[axis("FirstAxis => The description for the first axis")]
        FirstAxis,
        #[axis("SecondAxis => The description for the second axis")]
        SecondAxis,
    }

    trace!("Instantiating the aggregator struct and confirming defaults.");
    let aggregator = SimpleAxisExpander::default();

    info!("Confirm aggregator implements SystemMessageGoal with fallback text.");
    let goal: Cow<'_, str> = aggregator.system_message_goal();
    assert_eq!(goal, "Default system message goal");

    debug!("Confirm aggregator name is the enum name, as derived.");
    assert_eq!(aggregator.name(), "SimpleAxisExpander");

    trace!("Check aggregator.axes() returns all variants of the enum.");
    let axes = aggregator.axes();
    assert_eq!(axes.len(), 2, "Expected exactly two variants.");

    info!("Confirm each axis implements AxisName + AxisDescription correctly.");
    let axis0_name = axes[0].axis_name();
    let axis1_name = axes[1].axis_name();
    assert_eq!(axis0_name, "FirstAxis");
    assert_eq!(axis1_name, "SecondAxis");

    let axis0_desc = axes[0].axis_description();
    let axis1_desc = axes[1].axis_description();
    assert_eq!(axis0_desc, "The description for the first axis");
    assert_eq!(axis1_desc, "The description for the second axis");

    info!("check_missing_system_message_goal test complete.");
}

#[traced_test]
fn check_name_value_style_system_message_goal() {
    info!("Defining an enum with name-value style system_message_goal: #[system_message_goal = \"...\"]");

    #[derive(Debug, TokenExpansionAxis)]
    #[system_message_goal = "Name-Value Goal"]
    enum NameValueAxis {
        #[axis("Alpha => Describing alpha")]
        Alpha,
    }

    let aggregator = NameValueAxisExpander::default();
    let goal: Cow<'_, str> = aggregator.system_message_goal();
    assert_eq!(goal, "Name-Value Goal");

    info!("Check aggregator name is derived from the enum name with Expander suffix.");
    assert_eq!(aggregator.name(), "NameValueAxisExpander");

    let axes = aggregator.axes();
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].axis_name(), "Alpha");
    assert_eq!(axes[0].axis_description(), "Describing alpha");

    info!("check_name_value_style_system_message_goal test complete.");
}

#[traced_test]
fn check_parentheses_style_system_message_goal() {
    info!("Defining an enum with parentheses-style system_message_goal: #[system_message_goal(\"...\")]");

    #[derive(Debug, TokenExpansionAxis)]
    #[system_message_goal("Parentheses Goal")]
    enum ParenthesesAxis {
        #[axis("Beta => Description of beta axis")]
        Beta,
    }

    let aggregator = ParenthesesAxisExpander::default();
    let goal: Cow<'_, str> = aggregator.system_message_goal();
    assert_eq!(goal, "Parentheses Goal");

    info!("Check aggregator name is derived from the enum name with Expander suffix.");
    assert_eq!(aggregator.name(), "ParenthesesAxisExpander");

    let axes = aggregator.axes();
    assert_eq!(axes.len(), 1);
    assert_eq!(axes[0].axis_name(), "Beta");
    assert_eq!(axes[0].axis_description(), "Description of beta axis");

    info!("check_parentheses_style_system_message_goal test complete.");
}
