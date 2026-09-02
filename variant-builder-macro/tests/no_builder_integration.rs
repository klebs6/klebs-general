//! tests/variant_builder_no_builder_integration.rs

use variant_builder_macro::VariantBuilder;
use derive_builder::Builder;
use traced_test::*;
use tracing_setup::*;
use tracing::info;

// ────────────────────────────────────────────────────────────────
// Helper types
// ────────────────────────────────────────────────────────────────
//

#[derive(Debug, Clone, PartialEq)]
enum SimpleInnerEnum {
    Variant1,
    Variant2,
}

impl Default for SimpleInnerEnum {
    fn default() -> Self {
        Self::Variant1
    }
}

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(setter(into, strip_option))]
struct ComplexStruct {
    x: i32,
    #[builder(default)]
    y: Option<String>,
}

impl Default for ComplexStruct {
    fn default() -> Self {
        Self { x: 0, y: None }
    }
}

//
// ────────────────────────────────────────────────────────────────
// 1. MixedEnum (no_builder + builder variants)
// ────────────────────────────────────────────────────────────────
//

#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum MixedEnum {
    #[default]
    #[no_builder]
    Simple(SimpleInnerEnum),

    Complex(ComplexStruct),
}

#[traced_test]
fn mixed_simple_variant() {
    let inner = SimpleInnerEnum::Variant2;
    let result = MixedEnum::simple(inner.clone());
    info!(?result, "constructed MixedEnum::Simple");
    assert_eq!(result, MixedEnum::Simple(inner));
}

#[traced_test]
fn mixed_complex_variant() {
    let result = MixedEnum::complex(|b| {
        b.x(42).y("answer");
    });
    info!(?result, "constructed MixedEnum::Complex");
    assert_eq!(
        result,
        MixedEnum::Complex(ComplexStruct {
            x: 42,
            y: Some("answer".into())
        })
    );
}

#[traced_test]
fn mixed_default_variant() {
    let result = MixedEnum::default();
    info!(?result, "default MixedEnum");
    assert_eq!(result, MixedEnum::Simple(SimpleInnerEnum::default()));
}

#[test]
#[should_panic(expected = "Simulated failure")]
fn mixed_complex_builder_should_panic() {
    MixedEnum::complex(|_b| panic!("Simulated failure"));
}

//
// ────────────────────────────────────────────────────────────────
// 2. AllNoBuilder
// ────────────────────────────────────────────────────────────────
//

#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum AllNoBuilder {
    #[default]
    #[no_builder]
    A(SimpleInnerEnum),

    #[no_builder]
    B(SimpleInnerEnum),
}

#[traced_test]
fn all_no_builder_variant_a() {
    let result = AllNoBuilder::a(SimpleInnerEnum::Variant1);
    info!(?result, "constructed AllNoBuilder::A");
    assert_eq!(result, AllNoBuilder::A(SimpleInnerEnum::Variant1));
}

#[traced_test]
fn all_no_builder_variant_b() {
    let result = AllNoBuilder::b(SimpleInnerEnum::Variant2);
    info!(?result, "constructed AllNoBuilder::B");
    assert_eq!(result, AllNoBuilder::B(SimpleInnerEnum::Variant2));
}

#[traced_test]
fn all_no_builder_default_variant() {
    let result = AllNoBuilder::default();
    info!(?result, "default AllNoBuilder");
    assert_eq!(result, AllNoBuilder::A(SimpleInnerEnum::default()));
}

//
// ────────────────────────────────────────────────────────────────
// 3. AllBuilder
// ────────────────────────────────────────────────────────────────
//

#[derive(VariantBuilder, Debug, Clone, PartialEq)]
enum AllBuilder {
    #[default]
    First(ComplexStruct),
    Second(ComplexStruct),
}

#[traced_test]
fn all_builder_first_variant() {
    let result = AllBuilder::first(|b| {
        b.x(1).y("one");
    });
    info!(?result, "constructed AllBuilder::First");
    assert_eq!(
        result,
        AllBuilder::First(ComplexStruct {
            x: 1,
            y: Some("one".into())
        })
    );
}

#[traced_test]
fn all_builder_second_variant() {
    let result = AllBuilder::second(|b| {
        b.x(2).y("two");
    });
    info!(?result, "constructed AllBuilder::Second");
    assert_eq!(
        result,
        AllBuilder::Second(ComplexStruct {
            x: 2,
            y: Some("two".into())
        })
    );
}

#[traced_test]
fn all_builder_default_variant() {
    let result = AllBuilder::default();
    info!(?result, "default AllBuilder");
    assert_eq!(result, AllBuilder::First(ComplexStruct::default()));
}
