//! tests/nested_builder_integration.rs
//! Exhaustive checks for `#[nested_builder]`, `#[no_builder]`, and legacy
//! builder‑delegation helpers emitted by the VariantBuilder proc‑macro.

use variant_builder_macro::VariantBuilder;
use derive_builder::Builder;
use traced_test::*;
use tracing_setup::*;
use tracing::info;

// ────────────────────────────────────────────────────────────────
// Helper struct built with `derive_builder` (legacy builder path)
// ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(setter(into))]
struct LeafStruct {
    value: i32,
}
impl Default for LeafStruct {
    fn default() -> Self {
        Self { value: 0 }
    }
}

// ────────────────────────────────────────────────────────────────
// Innermost enum: demos `builder` + `no_builder` modes
// ────────────────────────────────────────────────────────────────
#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum LeafEnum {
    #[default]
    #[no_builder]
    Simple(i32),

    // builder‑delegation mode (uses `LeafStructBuilder`)
    Complex(LeafStruct),
}

// ────────────────────────────────────────────────────────────────
// Mid‑level enum: demos `#[nested_builder]`
// ────────────────────────────────────────────────────────────────
#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum MidEnum {
    #[default]
    #[nested_builder]
    Leaf(LeafEnum),

    #[no_builder]
    Number(u32),
}

// ────────────────────────────────────────────────────────────────
// Outermost enum: mixes all three modes again
// ────────────────────────────────────────────────────────────────
#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum OuterEnum {
    #[default]
    #[nested_builder]       // stacked builder
    Mid(MidEnum),

    // builder‑delegation mode
    TopStruct(TopStruct),

    #[no_builder]
    RawStr(String),
}

// Helper builder struct for `TopStruct` variant
#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(setter(into))]
struct TopStruct {
    label: &'static str,
}
impl Default for TopStruct {
    fn default() -> Self {
        Self { label: "default" }
    }
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[traced_test]
fn nested_builder_full_chain() {
    let tree = OuterEnum::mid(||                          // nested_builder #1
        MidEnum::leaf(||                                  // nested_builder #2
            LeafEnum::complex(|b| {                       // builder‑delegation
                b.value(42);
            })
        )
    );

    info!(?tree, "constructed full skill‑tree");

    assert_eq!(
        tree,
        OuterEnum::Mid(
            MidEnum::Leaf(
                LeafEnum::Complex(
                    LeafStruct { value: 42 }
                )
            )
        )
    );
}

#[traced_test]
fn outer_builder_variant() {
    let v = OuterEnum::top_struct(|b| {                   // builder‑delegation
        b.label("outer");
    });

    info!(?v, "constructed OuterEnum::TopStruct");
    assert_eq!(v, OuterEnum::TopStruct(TopStruct { label: "outer" }));
}

#[traced_test]
fn outer_no_builder_variant() {
    let v = OuterEnum::raw_str("hello".into());           // no_builder path
    info!(?v, "constructed OuterEnum::RawStr");
    assert_eq!(v, OuterEnum::RawStr("hello".into()));
}

#[traced_test]
fn mid_no_builder_variant() {
    let m = MidEnum::number(7);                           // no_builder path
    info!(?m, "constructed MidEnum::Number");
    assert_eq!(m, MidEnum::Number(7));
}

#[traced_test]
fn leaf_no_builder_variant() {
    let l = LeafEnum::simple(3);                          // no_builder path
    info!(?l, "constructed LeafEnum::Simple");
    assert_eq!(l, LeafEnum::Simple(3));
}

#[traced_test]
fn default_variants_chain() {
    // OuterEnum::default() should recurse through default variants
    let default_tree = OuterEnum::default();
    info!(?default_tree, "default tree produced");

    assert_eq!(
        default_tree,
        OuterEnum::Mid(
            MidEnum::Leaf(
                LeafEnum::Simple(0)   // LeafStruct's default is 0, but LeafEnum's default is Simple(0)
            )
        )
    );
}

#[test]
#[should_panic(expected = "Simulated failure")]
fn builder_closure_propagates_panic() {
    // Panic should propagate out of builder‑delegation helper
    OuterEnum::top_struct(|_b| panic!("Simulated failure"));
}

