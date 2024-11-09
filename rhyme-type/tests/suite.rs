use rhyme_type::*;
use named_item::AIDescriptor;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::Rng;
use serde_json;

#[test]
fn test_rhyme_position_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let position: RhymePosition = rng.gen();
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
fn test_rhyme_position_ai() {
    let position = RhymePosition::End;
    assert_eq!(
        position.ai(),
        "The rhymes should occur at the end of lines."
    );
}

#[test]
fn test_rhyme_quality_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let quality: RhymeQuality = rng.gen();
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
        quality.ai(),
        "Use perfect rhymes, where both consonant and vowel sounds match exactly."
    );
}

#[test]
fn test_rhyme_scheme_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let scheme: RhymeScheme = rng.gen();
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
        scheme.ai(),
        "Follow a couplet rhyme scheme (AABB)."
    );

    let custom_scheme = RhymeScheme::Custom("ABCD".to_string());
    assert_eq!(
        custom_scheme.ai(),
        "Follow a custom rhyme scheme: ABCD."
    );
}

#[test]
fn test_rhyme_stress_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let stress: RhymeStress = rng.gen();
    assert!(matches!(
        stress,
        RhymeStress::Masculine | RhymeStress::Feminine | RhymeStress::Triple
    ));
}

#[test]
fn test_rhyme_stress_ai() {
    let stress = RhymeStress::Masculine;
    assert_eq!(
        stress.ai(),
        "The rhymes should be masculine, rhyming the final stressed syllable."
    );
}

#[test]
fn test_special_rhyme_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let special: SpecialRhyme = rng.gen();
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
        special.ai(),
        "Use cross rhymes, rhyming in a cross pattern like ABBA."
    );
}

#[test]
fn test_rhyme_type_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let rhyme_type: RhymeType = rng.gen();
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

    assert_eq!(rhyme_type.ai(), expected);
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

    let custom_scheme = RhymeScheme::Custom("ABCD".to_string());
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
    let rhyme_type1: RhymeType = rng1.gen();
    let rhyme_type2: RhymeType = rng2.gen();
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
        let desc = quality.ai();
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
        let desc = position.ai();
        assert!(!desc.is_empty());
    }

    let stresses = [
        RhymeStress::Masculine,
        RhymeStress::Feminine,
        RhymeStress::Triple,
    ];
    for stress in &stresses {
        let desc = stress.ai();
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
        let desc = special.ai();
        assert!(!desc.is_empty());
    }
}

#[test]
fn test_rhyme_type_fields() {
    let mut rng = StdRng::seed_from_u64(42);
    let rhyme_type: RhymeType = rng.gen();

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


