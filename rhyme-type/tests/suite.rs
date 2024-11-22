use rhyme_type::*;
use rand_construct::*;
use ai_descriptor::*;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json;

#[test]
fn test_rhyme_position_distribution() {
    let position = RhymePosition::random();
    assert!(matches!(
        position,
        RhymePosition::End
            | RhymePosition::Internal
            | RhymePosition::Head
            | RhymePosition::Interlaced
            | RhymePosition::Linked
            | RhymePosition::Holorhyme
            | RhymePosition::Tail
    ));
}

#[test]
fn test_rhyme_position_text() {
    let position = RhymePosition::End;
    assert_eq!(
        position.text(),
        "The rhymes should occur at the end of lines."
    );
}

#[test]
fn test_rhyme_quality_distribution() {
    let quality = RhymeQuality::random();
    assert!(matches!(
        quality,
        RhymeQuality::Perfect
            | RhymeQuality::Slant
            | RhymeQuality::Eye
            | RhymeQuality::Identical
            | RhymeQuality::Rich
            | RhymeQuality::Wrenched
            | RhymeQuality::Light
            | RhymeQuality::MultiSyllabic
            | RhymeQuality::Compound
            | RhymeQuality::Broken
            | RhymeQuality::Macaronic
    ));
}

#[test]
fn test_rhyme_quality_ai() {
    let quality = RhymeQuality::Perfect;
    assert_eq!(
        quality.text(),
        "Use perfect rhymes, where both consonant and vowel sounds match exactly."
    );
}

#[test]
fn test_rhyme_scheme_distribution() {
    let scheme = RhymeScheme::random();
    match scheme {
        RhymeScheme::Custom(ref pattern) => {
            assert!(["ABCD", "AABCCB", "ABACAD"].contains(&pattern.as_str()));
        }
        _ => {
            // Check that it's one of the standard schemes
            assert!(matches!(
                scheme,
                RhymeScheme::Couplet
                    | RhymeScheme::Alternate
                    | RhymeScheme::Enclosed
                    | RhymeScheme::Chain
                    | RhymeScheme::Monorhyme
                    | RhymeScheme::Limerick
                    | RhymeScheme::Villanelle
                    | RhymeScheme::SonnetShakespearean
                    | RhymeScheme::SonnetPetrarchan
                    | RhymeScheme::TerzaRima
            ));
        }
    }
}

#[test]
fn test_rhyme_scheme_ai() {
    let scheme = RhymeScheme::Couplet;
    assert_eq!(
        scheme.text(),
        "Follow a couplet rhyme scheme (AABB)."
    );

    let custom_scheme = RhymeScheme::Custom(CustomRhymeScheme::from("ABCD"));
    assert_eq!(
        custom_scheme.text(),
        "Follows a custom rhyme scheme: ABCD."
    );
}

#[test]
fn test_rhyme_stress_distribution() {
    let stress = RhymeStress::random();
    assert!(matches!(
        stress,
        RhymeStress::Masculine | RhymeStress::Feminine | RhymeStress::Triple
    ));
}

#[test]
fn test_rhyme_stress_ai() {
    let stress = RhymeStress::Masculine;
    assert_eq!(
        stress.text(),
        "The rhymes should be masculine, rhyming the final stressed syllable."
    );
}

#[test]
fn test_special_rhyme_distribution() {
    let special = SpecialRhyme::random();
    assert!(matches!(
        special,
        SpecialRhyme::Cross
            | SpecialRhyme::Sporadic
            | SpecialRhyme::FreeVerse
            | SpecialRhyme::BlankVerse
            | SpecialRhyme::Enjambment
            | SpecialRhyme::Acrostic
    ));
}

#[test]
fn test_special_rhyme_ai() {
    let special = SpecialRhyme::Cross;
    assert_eq!(
        special.text(),
        "Use cross rhymes, rhyming in a cross pattern like ABBA."
    );
}

#[test]
fn test_rhyme_type_distribution() {
    let rhyme_type = RhymeType::random();
    // Since fields are optional, we can only assert that `quality` is valid
    assert!(matches!(
        rhyme_type.rhyme_quality(),
        RhymeQuality::Perfect
            | RhymeQuality::Slant
            | RhymeQuality::Eye
            | RhymeQuality::Identical
            | RhymeQuality::Rich
            | RhymeQuality::Wrenched
            | RhymeQuality::Light
            | RhymeQuality::MultiSyllabic
            | RhymeQuality::Compound
            | RhymeQuality::Broken
            | RhymeQuality::Macaronic
    ));
}

#[test]
fn test_rhyme_type_ai() {
    let rhyme_type 
        = RhymeType::builder()
        .position(RhymePosition::End)
        .stress(RhymeStress::Masculine)
        .scheme(RhymeScheme::Couplet)
        .special(SpecialRhyme::Cross)
        .build();

    let expected = "Use perfect rhymes, where both consonant and vowel sounds match exactly. \
                    The rhymes should occur at the end of lines. \
The rhymes should be masculine, rhyming the final stressed syllable. \
Follow a couplet rhyme scheme (AABB). \
Use cross rhymes, rhyming in a cross pattern like ABBA.";

    assert_eq!(rhyme_type.ai_text(), expected);
}

#[test]
fn test_rhyme_type_serialization() {
    let rhyme_type = RhymeType::builder()
        .position(RhymePosition::End)
        .stress(RhymeStress::Masculine)
        .scheme(RhymeScheme::Couplet)
        .special(SpecialRhyme::Cross)
        .build();

    let serialized = serde_json::to_string(&rhyme_type).unwrap();
    let deserialized: RhymeType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(rhyme_type, deserialized);
}

#[test]
fn test_rhyme_scheme_serialization() {
    let scheme = RhymeScheme::Couplet;
    let serialized = serde_json::to_string(&scheme).unwrap();
    let deserialized: RhymeScheme = serde_json::from_str(&serialized).unwrap();
    assert_eq!(scheme, deserialized);

    let custom_scheme = RhymeScheme::Custom(CustomRhymeScheme::from("ABCD"));
    let serialized_custom = serde_json::to_string(&custom_scheme).unwrap();
    let deserialized_custom: RhymeScheme = serde_json::from_str(&serialized_custom).unwrap();
    assert_eq!(custom_scheme, deserialized_custom);
}

#[test]
fn test_rhyme_quality_serialization() {
    let quality = RhymeQuality::Perfect;
    let serialized = serde_json::to_string(&quality).unwrap();
    let deserialized: RhymeQuality = serde_json::from_str(&serialized).unwrap();
    assert_eq!(quality, deserialized);
}

#[test]
fn test_rhyme_position_serialization() {
    let position = RhymePosition::End;
    let serialized = serde_json::to_string(&position).unwrap();
    let deserialized: RhymePosition = serde_json::from_str(&serialized).unwrap();
    assert_eq!(position, deserialized);
}

#[test]
fn test_rhyme_stress_serialization() {
    let stress = RhymeStress::Masculine;
    let serialized = serde_json::to_string(&stress).unwrap();
    let deserialized: RhymeStress = serde_json::from_str(&serialized).unwrap();
    assert_eq!(stress, deserialized);
}

#[test]
fn test_special_rhyme_serialization() {
    let special = SpecialRhyme::Cross;
    let serialized = serde_json::to_string(&special).unwrap();
    let deserialized: SpecialRhyme = serde_json::from_str(&serialized).unwrap();
    assert_eq!(special, deserialized);
}

#[test]
fn test_rhyme_type_equality() {

    let rhyme_type1 
        = RhymeType::builder()
        .position(RhymePosition::End)
        .stress(RhymeStress::Masculine)
        .scheme(RhymeScheme::Couplet)
        .special(SpecialRhyme::Cross)
        .build();

    let rhyme_type2 = rhyme_type1.clone();
    assert_eq!(rhyme_type1, rhyme_type2);
}

#[test]
fn test_rhyme_type_randomness() {
    let mut rng1 = StdRng::seed_from_u64(1);
    let mut rng2 = StdRng::seed_from_u64(1);
    let rhyme_type1 = RhymeType::random_with_rng(&mut rng1);
    let rhyme_type2 = RhymeType::random_with_rng(&mut rng2);
    assert_eq!(rhyme_type1, rhyme_type2);
}

#[test]
fn test_ais_not_empty() {
    // Test that all ais return non-empty strings
    let qualities = [
        RhymeQuality::Perfect,
        RhymeQuality::Slant,
        RhymeQuality::Eye,
        RhymeQuality::Identical,
        RhymeQuality::Rich,
        RhymeQuality::Wrenched,
        RhymeQuality::Light,
        RhymeQuality::MultiSyllabic,
        RhymeQuality::Compound,
        RhymeQuality::Broken,
        RhymeQuality::Macaronic,
    ];
    for quality in &qualities {
        let desc = quality.text();
        assert!(!desc.is_empty());
    }

    let positions = [
        RhymePosition::End,
        RhymePosition::Internal,
        RhymePosition::Head,
        RhymePosition::Interlaced,
        RhymePosition::Linked,
        RhymePosition::Holorhyme,
        RhymePosition::Tail,
    ];
    for position in &positions {
        let desc = position.text();
        assert!(!desc.is_empty());
    }

    let stresses = [
        RhymeStress::Masculine,
        RhymeStress::Feminine,
        RhymeStress::Triple,
    ];
    for stress in &stresses {
        let desc = stress.text();
        assert!(!desc.is_empty());
    }

    let specials = [
        SpecialRhyme::Cross,
        SpecialRhyme::Sporadic,
        SpecialRhyme::FreeVerse,
        SpecialRhyme::BlankVerse,
        SpecialRhyme::Enjambment,
        SpecialRhyme::Acrostic,
    ];
    for special in &specials {
        let desc = special.text();
        assert!(!desc.is_empty());
    }
}

#[test]
fn test_rhyme_type_fields() {
    let rhyme_type = RhymeType::random();

    // Test that quality is always present and valid
    assert!(matches!(
        rhyme_type.rhyme_quality(),
        RhymeQuality::Perfect
            | RhymeQuality::Slant
            | RhymeQuality::Eye
            | RhymeQuality::Identical
            | RhymeQuality::Rich
            | RhymeQuality::Wrenched
            | RhymeQuality::Light
            | RhymeQuality::MultiSyllabic
            | RhymeQuality::Compound
            | RhymeQuality::Broken
            | RhymeQuality::Macaronic
    ));

    // Test optional fields if they are Some
    if let Some(position) = rhyme_type.rhyme_position() {
        assert!(matches!(
            position,
            RhymePosition::End
                | RhymePosition::Internal
                | RhymePosition::Head
                | RhymePosition::Interlaced
                | RhymePosition::Linked
                | RhymePosition::Holorhyme
                | RhymePosition::Tail
        ));
    }

    if let Some(stress) = rhyme_type.rhyme_stress() {
        assert!(matches!(
            stress,
            RhymeStress::Masculine | RhymeStress::Feminine | RhymeStress::Triple
        ));
    }

    if let Some(ref scheme) = rhyme_type.rhyme_scheme() {
        match scheme {
            RhymeScheme::Custom(ref pattern) => {
                assert!(["ABCD", "AABCCB", "ABACAD"].contains(&pattern.as_str()));
            }
            _ => {
                assert!(matches!(
                    scheme,
                    RhymeScheme::Couplet
                        | RhymeScheme::Alternate
                        | RhymeScheme::Enclosed
                        | RhymeScheme::Chain
                        | RhymeScheme::Monorhyme
                        | RhymeScheme::Limerick
                        | RhymeScheme::Villanelle
                        | RhymeScheme::SonnetShakespearean
                        | RhymeScheme::SonnetPetrarchan
                        | RhymeScheme::TerzaRima
                ));
            }
        }
    }

    if let Some(special) = rhyme_type.rhyme_special() {
        assert!(matches!(
            special,
            SpecialRhyme::Cross
                | SpecialRhyme::Sporadic
                | SpecialRhyme::FreeVerse
                | SpecialRhyme::BlankVerse
                | SpecialRhyme::Enjambment
                | SpecialRhyme::Acrostic
        ));
    }
}
