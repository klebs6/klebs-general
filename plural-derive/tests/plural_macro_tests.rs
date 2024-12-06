#![allow(dead_code)]
use plural_derive::Plural;
use plural_trait::*;

#[derive(Plural, Debug)]
pub enum TestEnum {
    #[plural("alcaic stanzas")]
    AlcaicStanza,            // Custom pluralization

    #[plural("ballads")]
    Ballad,                  // Custom pluralization

    #[plural("burns stanzas")]
    BurnsStanza,             // Custom pluralization

    BlankVerse,              // Default pluralization: "blank verses"
    Couplet,                 // Default pluralization: "couplets"
    Haiku,                   // Default pluralization: "haikus"

    #[plural("irregular plural")]
    Irregular,               // Irregular custom plural

    #[plural("with special cases")]
    SpecialCase,             // Custom plural with spaces
}

// Empty enum for testing compile-time behavior
#[derive(Plural, Debug)]
pub enum EmptyEnum {}

#[test]
fn test_custom_pluralization() {
    let form = TestEnum::AlcaicStanza;
    assert_eq!(form.plural(), "alcaic stanzas");

    let form = TestEnum::Ballad;
    assert_eq!(form.plural(), "ballads");

    let form = TestEnum::BurnsStanza;
    assert_eq!(form.plural(), "burns stanzas");

    let form = TestEnum::Irregular;
    assert_eq!(form.plural(), "irregular plural");

    let form = TestEnum::SpecialCase;
    assert_eq!(form.plural(), "with special cases");
}

#[test]
fn test_default_pluralization() {
    let form = TestEnum::BlankVerse;
    assert_eq!(form.plural(), "blank verses");

    let form = TestEnum::Couplet;
    assert_eq!(form.plural(), "couplets");

    let form = TestEnum::Haiku;
    assert_eq!(form.plural(), "haikus");
}

#[test]
fn test_camelcase_to_lowercase() {
    #[derive(Plural, Debug)]
    enum CamelCaseEnum {
        SimpleCase,           // Default pluralization: "simple cases"
        MixedCaseName,        // Default pluralization: "mixed case names"
        AnotherCaseExample,   // Default pluralization: "another case examples"
    }

    let form = CamelCaseEnum::SimpleCase;
    assert_eq!(form.plural(), "simple cases");

    let form = CamelCaseEnum::MixedCaseName;
    assert_eq!(form.plural(), "mixed case names");

    let form = CamelCaseEnum::AnotherCaseExample;
    assert_eq!(form.plural(), "another case examples");
}

#[test]
fn test_empty_enum() {
    // Verify that the macro handles empty enums without generating invalid code.
    let result = std::panic::catch_unwind(|| {
        let _ = std::mem::size_of::<EmptyEnum>(); // Ensure the type exists but is uninhabited.
    });
    assert!(result.is_ok());
}

#[test]
fn test_no_plural_attribute_defaults() {
    #[derive(Plural, Debug)]
    enum NoCustomPlural {
        AlphaCase, // Default: "alpha cases"
        BetaForm,  // Default: "beta forms"
    }

    let form = NoCustomPlural::AlphaCase;
    assert_eq!(form.plural(), "alpha cases");

    let form = NoCustomPlural::BetaForm;
    assert_eq!(form.plural(), "beta forms");
}

#[test]
fn test_multiple_enums_in_same_scope() {
    #[derive(Plural, Debug)]
    enum EnumOne {
        FirstVariant, // Default: "first variants"
        SecondVariant,
    }

    #[derive(Plural, Debug)]
    enum EnumTwo {
        ThirdVariant, // Default: "third variants"
        FourthVariant,
    }

    let form = EnumOne::FirstVariant;
    assert_eq!(form.plural(), "first variants");

    let form = EnumTwo::ThirdVariant;
    assert_eq!(form.plural(), "third variants");
}

#[test]
fn test_special_characters_in_custom_plural() {
    #[derive(Plural, Debug)]
    enum SpecialCharsEnum {
        #[plural("special-characters")]
        DashVariant,

        #[plural("special characters with spaces")]
        SpaceVariant,

        #[plural("special.characters")]
        DotVariant,
    }

    let form = SpecialCharsEnum::DashVariant;
    assert_eq!(form.plural(), "special-characters");

    let form = SpecialCharsEnum::SpaceVariant;
    assert_eq!(form.plural(), "special characters with spaces");

    let form = SpecialCharsEnum::DotVariant;
    assert_eq!(form.plural(), "special.characters");
}
