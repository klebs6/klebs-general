use variant_builder::VariantBuilder;
use derive_builder::Builder; // Using `derive_builder` for test struct builders

// Test Enum with VariantBuilder
#[derive(VariantBuilder, Debug, Clone, PartialEq)]
pub enum TestEnum {
    VariantA(TestVariantABuilder),
    VariantB(TestVariantBBuilder),
    VariantC(TestVariantCBuilder),
}

#[derive(Debug, Clone, PartialEq, Builder)]
#[builder(setter(into, strip_option))]
pub struct TestVariantABuilder {
    #[builder(default = "\"DefaultA\".to_string()")] // Explicitly set the default
    pub field_a: String,
    #[builder(default = "0")] // Explicitly set the default
    pub field_b: i32,
}

impl Default for TestVariantABuilder {
    fn default() -> Self {
        Self {
            field_a: "DefaultA".to_string(),
            field_b: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Builder, Default)]
#[builder(setter(into, strip_option))]
pub struct TestVariantBBuilder {
    pub field_x: f64,
    pub field_y: bool,
}

#[derive(Debug, Clone, PartialEq, Builder, Default)]
#[builder(setter(into, strip_option))]
pub struct TestVariantCBuilder {
    pub field_m: i32,
    #[builder(default)]
    pub field_n: Option<String>,
}

// Test Suite
#[test]
fn test_generated_methods_variant_a() {
    let result = TestEnum::variant_a(|builder| {
        builder
            .field_a("Hello".to_string())
            .field_b(42);
    });

    assert_eq!(
        result,
        TestEnum::VariantA(TestVariantABuilder {
            field_a: "Hello".into(),
            field_b: 42,
        })
    );
}

#[test]
fn test_generated_methods_variant_b() {
    let result = TestEnum::variant_b(|builder| {
        builder
            .field_x(3.14)
            .field_y(true);
    });

    assert_eq!(
        result,
        TestEnum::VariantB(TestVariantBBuilder {
            field_x: 3.14,
            field_y: true,
        })
    );
}

#[test]
fn test_generated_methods_variant_c() {
    let result = TestEnum::variant_c(|builder| {
        builder
            .field_m(-100)
            .field_n("Optional value".to_string());
    });

    assert_eq!(
        result,
        TestEnum::VariantC(TestVariantCBuilder {
            field_m: -100,
            field_n: Some("Optional value".into()),
        })
    );
}

#[test]
fn test_default_builder_usage() {
    // Using default values without modifying the builder
    let result = TestEnum::variant_a(|_builder| {});

    println!("Result: {:?}", result);

    assert_eq!(
        result,
        TestEnum::VariantA(TestVariantABuilder::default())
    );
}

#[test]
fn test_test_variant_a_default() {
    let default = TestVariantABuilder::default();
    assert_eq!(
        default,
        TestVariantABuilder {
            field_a: "DefaultA".to_string(),
            field_b: 0,
        }
    );
}


#[test]
fn test_partial_builder_usage() {
    // Using only some of the fields in the builder
    let result = TestEnum::variant_c(|builder| {
        builder.field_m(99);
    });

    assert_eq!(
        result,
        TestEnum::VariantC(TestVariantCBuilder {
            field_m: 99,
            field_n: None, // Default value
        })
    );
}

#[test]
#[should_panic(expected = "Simulated failure")]
fn test_invalid_build_should_panic() {
    TestEnum::variant_a(|_builder| panic!("Simulated failure"));
}

#[test]
fn test_macro_handles_complex_enum() {
    // Ensure the macro works for enums with many variants
    #[derive(VariantBuilder, Debug, Clone, PartialEq)]
    pub enum ComplexEnum {
        A(TestVariantABuilder),
        B(TestVariantBBuilder),
        C(TestVariantCBuilder),
    }

    let result = ComplexEnum::a(|builder| {
        builder.field_a("Complex".to_string()).field_b(123);
    });

    assert_eq!(
        result,
        ComplexEnum::A(TestVariantABuilder {
            field_a: "Complex".into(),
            field_b: 123,
        })
    );
}
