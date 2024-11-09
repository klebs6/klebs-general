use lyrical_meter::*;
use named_item::AIDescriptor;
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand::Rng;
use serde_json;

#[test]
fn test_metrical_foot_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let foot: MetricalFoot = rng.gen();
    assert!(matches!(
        foot,
        MetricalFoot::Iamb
            | MetricalFoot::Trochee
            | MetricalFoot::Anapest
            | MetricalFoot::Dactyl
            | MetricalFoot::Spondee
            | MetricalFoot::Pyrrhic
            | MetricalFoot::Amphibrach
            | MetricalFoot::Amphimacer
            | MetricalFoot::Bacchic
            | MetricalFoot::Cretic
            | MetricalFoot::Antibacchius
            | MetricalFoot::Molossus
            | MetricalFoot::Tribrach
            | MetricalFoot::Choriamb
            | MetricalFoot::IonicAMinore
            | MetricalFoot::IonicAMajore
            | MetricalFoot::Aeolic
    ));
}

#[test]
fn test_metrical_foot_ai_descriptor() {
    let foot = MetricalFoot::Iamb;
    assert_eq!(
        foot.ai(),
        "Use iambic meter, with unstressed-stressed syllables."
    );
}

#[test]
fn test_line_length_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let length: LineLength = rng.gen();
    assert!(matches!(
        length,
        LineLength::Monometer
            | LineLength::Dimeter
            | LineLength::Trimeter
            | LineLength::Tetrameter
            | LineLength::Pentameter
            | LineLength::Hexameter
            | LineLength::Heptameter
            | LineLength::Octameter
            | LineLength::Nonameter
            | LineLength::Decameter
    ));
}

#[test]
fn test_line_length_ai_descriptor() {
    let length = LineLength::Pentameter;
    assert_eq!(
        length.ai(),
        "Each line should have five feet (pentameter)."
    );
}

#[test]
fn test_other_meter_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let other_meter: OtherMeter = rng.gen();
    assert!(matches!(
        other_meter,
        OtherMeter::ClimbingRhyme
            | OtherMeter::FallingRhyme
            | OtherMeter::MixedMeter
            | OtherMeter::FreeVerse
            | OtherMeter::BlankVerse
    ));
}

#[test]
fn test_other_meter_ai_descriptor() {
    let other_meter = OtherMeter::FreeVerse;
    assert_eq!(
        other_meter.ai(),
        "Write in free verse, without a consistent meter or rhyme scheme."
    );
}

#[test]
fn test_lyrical_meter_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let lyrical_meter: LyricalMeter = rng.gen();
    // We can't predict the exact values, but we can check that the foot is valid
    assert!(matches!(
        lyrical_meter.foot(),
        MetricalFoot::Iamb
            | MetricalFoot::Trochee
            | MetricalFoot::Anapest
            | MetricalFoot::Dactyl
            | MetricalFoot::Spondee
            | MetricalFoot::Pyrrhic
            | MetricalFoot::Amphibrach
            | MetricalFoot::Amphimacer
            | MetricalFoot::Bacchic
            | MetricalFoot::Cretic
            | MetricalFoot::Antibacchius
            | MetricalFoot::Molossus
            | MetricalFoot::Tribrach
            | MetricalFoot::Choriamb
            | MetricalFoot::IonicAMinore
            | MetricalFoot::IonicAMajore
            | MetricalFoot::Aeolic
    ));
}

#[test]
fn test_lyrical_meter_ai_descriptor() {
    let lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Dactyl)
        .length(LineLength::Hexameter)
        .build();

    let expected = "Use dactylic meter, with stressed-unstressed-unstressed syllables. Each line should have six feet (hexameter).";
    assert_eq!(lyrical_meter.ai(), expected);
}

#[test]
fn test_meter_distribution() {
    let mut rng = StdRng::seed_from_u64(42);
    let meter: Meter = rng.gen();
    match meter {
        Meter::Standard(ref lyrical_meter) => {
            assert!(matches!(
                lyrical_meter.foot(),
                MetricalFoot::Iamb
                    | MetricalFoot::Trochee
                    | MetricalFoot::Anapest
                    | MetricalFoot::Dactyl
                    | MetricalFoot::Spondee
                    | MetricalFoot::Pyrrhic
                    | MetricalFoot::Amphibrach
                    | MetricalFoot::Amphimacer
                    | MetricalFoot::Bacchic
                    | MetricalFoot::Cretic
                    | MetricalFoot::Antibacchius
                    | MetricalFoot::Molossus
                    | MetricalFoot::Tribrach
                    | MetricalFoot::Choriamb
                    | MetricalFoot::IonicAMinore
                    | MetricalFoot::IonicAMajore
                    | MetricalFoot::Aeolic
            ));
        }
        Meter::Other(ref other_meter) => {
            assert!(matches!(
                other_meter,
                OtherMeter::ClimbingRhyme
                    | OtherMeter::FallingRhyme
                    | OtherMeter::MixedMeter
                    | OtherMeter::FreeVerse
                    | OtherMeter::BlankVerse
            ));
        }
    }
}

#[test]
fn test_meter_ai_descriptor() {
    let meter = Meter::Other(OtherMeter::BlankVerse);
    assert_eq!(
        meter.ai(),
        "Write in blank verse, using unrhymed iambic pentameter."
    );
}

#[test]
fn test_lyrical_meter_builder() {
    let lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Anapest)
        .length(LineLength::Tetrameter)
        .build();

    assert_eq!(lyrical_meter.foot(), &MetricalFoot::Anapest);
    assert_eq!(lyrical_meter.length(), Some(&LineLength::Tetrameter));
}

#[test]
fn test_lyrical_meter_getters_setters() {
    let mut lyrical_meter = LyricalMeter::default();
    lyrical_meter.set_foot(MetricalFoot::Trochee);
    lyrical_meter.set_length(Some(LineLength::Pentameter));

    assert_eq!(lyrical_meter.foot(), &MetricalFoot::Trochee);
    assert_eq!(lyrical_meter.length(), Some(&LineLength::Pentameter));
}

#[test]
fn test_meter_builder_standard() {
    let meter = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Dactyl)
            .length(LineLength::Hexameter)
            .build(),
    );

    if let Meter::Standard(ref lyrical_meter) = meter {
        assert_eq!(lyrical_meter.foot(), &MetricalFoot::Dactyl);
        assert_eq!(lyrical_meter.length(), Some(&LineLength::Hexameter));
    } else {
        panic!("Expected Meter::Standard variant");
    }
}

#[test]
fn test_meter_builder_other() {
    let meter = Meter::Other(OtherMeter::FreeVerse);

    if let Meter::Other(ref other_meter) = meter {
        assert_eq!(other_meter, &OtherMeter::FreeVerse);
    } else {
        panic!("Expected Meter::Other variant");
    }
}

#[test]
fn test_other_meter_methods() {
    let other_meter = OtherMeter::FreeVerse;
    assert!(other_meter.is_free_verse());
    assert!(!other_meter.is_blank_verse());

    let other_meter = OtherMeter::BlankVerse;
    assert!(other_meter.is_blank_verse());
    assert!(!other_meter.is_free_verse());
}

#[test]
fn test_meter_is_same_type() {
    let meter1 = Meter::Standard(LyricalMeter::default());
    let meter2 = Meter::Other(OtherMeter::MixedMeter);
    let meter3 = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Trochee)
            .build(),
    );

    assert!(meter1.is_same_type(&meter3));
    assert!(!meter1.is_same_type(&meter2));
}

#[test]
fn test_meter_as_standard() {
    let meter = Meter::Standard(LyricalMeter::default());
    assert!(meter.as_standard().is_some());
    assert!(meter.as_other().is_none());
}

#[test]
fn test_meter_as_other() {
    let meter = Meter::Other(OtherMeter::FreeVerse);
    assert!(meter.as_other().is_some());
    assert!(meter.as_standard().is_none());
}

#[test]
fn test_metrical_foot_serialization() {
    let foot = MetricalFoot::Iamb;
    let serialized = serde_json::to_string(&foot).unwrap();
    let deserialized: MetricalFoot = serde_json::from_str(&serialized).unwrap();
    assert_eq!(foot, deserialized);
}

#[test]
fn test_line_length_serialization() {
    let length = LineLength::Pentameter;
    let serialized = serde_json::to_string(&length).unwrap();
    let deserialized: LineLength = serde_json::from_str(&serialized).unwrap();
    assert_eq!(length, deserialized);
}

#[test]
fn test_other_meter_serialization() {
    let other_meter = OtherMeter::FreeVerse;
    let serialized = serde_json::to_string(&other_meter).unwrap();
    let deserialized: OtherMeter = serde_json::from_str(&serialized).unwrap();
    assert_eq!(other_meter, deserialized);
}

#[test]
fn test_lyrical_meter_serialization() {
    let lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Dactyl)
        .length(LineLength::Hexameter)
        .build();
    let serialized = serde_json::to_string(&lyrical_meter).unwrap();
    let deserialized: LyricalMeter = serde_json::from_str(&serialized).unwrap();
    assert_eq!(lyrical_meter, deserialized);
}

#[test]
fn test_meter_serialization() {
    let meter = Meter::Other(OtherMeter::BlankVerse);
    let serialized = serde_json::to_string(&meter).unwrap();
    let deserialized: Meter = serde_json::from_str(&serialized).unwrap();
    assert_eq!(meter, deserialized);

    let meter = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Anapest)
            .length(LineLength::Trimeter)
            .build(),
    );
    let serialized = serde_json::to_string(&meter).unwrap();
    let deserialized: Meter = serde_json::from_str(&serialized).unwrap();
    assert_eq!(meter, deserialized);
}

#[test]
fn test_display_implementations() {
    let foot = MetricalFoot::Iamb;
    assert_eq!(format!("{:?}", foot), "Iamb");

    let length = LineLength::Pentameter;
    assert_eq!(format!("{:?}", length), "Pentameter");

    let other_meter = OtherMeter::FreeVerse;
    assert_eq!(
        format!("{}", other_meter),
        "Write in free verse, without a consistent meter or rhyme scheme."
    );

    let lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Dactyl)
        .length(LineLength::Hexameter)
        .build();
    assert_eq!(format!("{}", lyrical_meter), "Dactyl in Hexameter");

    let meter = Meter::Standard(lyrical_meter);
    assert_eq!(format!("{}", meter), "Dactyl in Hexameter");

    let meter = Meter::Other(OtherMeter::BlankVerse);
    assert_eq!(
        format!("{}", meter),
        "Write in blank verse, using unrhymed iambic pentameter."
    );
}

#[test]
fn test_default_implementations() {
    let lyrical_meter = LyricalMeter::default();
    assert_eq!(lyrical_meter.foot(), &MetricalFoot::Iamb);
    assert_eq!(lyrical_meter.length(), None);

    let meter_builder = MeterBuilder::default();
    let meter = meter_builder.build();
    if let Meter::Standard(ref lyrical_meter) = meter {
        assert_eq!(lyrical_meter.foot(), &MetricalFoot::Iamb);
        assert_eq!(lyrical_meter.length(), None);
    } else {
        panic!("Expected Meter::Standard variant");
    }
}

#[test]
fn test_random_generation_consistency() {
    let mut rng1 = StdRng::seed_from_u64(42);
    let mut rng2 = StdRng::seed_from_u64(42);

    let foot1: MetricalFoot = rng1.gen();
    let foot2: MetricalFoot = rng2.gen();
    assert_eq!(foot1, foot2);

    let length1: LineLength = rng1.gen();
    let length2: LineLength = rng2.gen();
    assert_eq!(length1, length2);

    let other_meter1: OtherMeter = rng1.gen();
    let other_meter2: OtherMeter = rng2.gen();
    assert_eq!(other_meter1, other_meter2);

    let lyrical_meter1: LyricalMeter = rng1.gen();
    let lyrical_meter2: LyricalMeter = rng2.gen();
    assert_eq!(lyrical_meter1, lyrical_meter2);

    let meter1: Meter = rng1.gen();
    let meter2: Meter = rng2.gen();
    assert_eq!(meter1, meter2);
}

#[test]
fn test_ai_descriptors_not_empty() {
    let feet = [
        MetricalFoot::Iamb,
        MetricalFoot::Trochee,
        MetricalFoot::Anapest,
        MetricalFoot::Dactyl,
        MetricalFoot::Spondee,
        MetricalFoot::Pyrrhic,
        MetricalFoot::Amphibrach,
        MetricalFoot::Amphimacer,
        MetricalFoot::Bacchic,
        MetricalFoot::Cretic,
        MetricalFoot::Antibacchius,
        MetricalFoot::Molossus,
        MetricalFoot::Tribrach,
        MetricalFoot::Choriamb,
        MetricalFoot::IonicAMinore,
        MetricalFoot::IonicAMajore,
        MetricalFoot::Aeolic,
    ];
    for foot in &feet {
        assert!(!foot.ai().is_empty());
    }

    let lengths = [
        LineLength::Monometer,
        LineLength::Dimeter,
        LineLength::Trimeter,
        LineLength::Tetrameter,
        LineLength::Pentameter,
        LineLength::Hexameter,
        LineLength::Heptameter,
        LineLength::Octameter,
        LineLength::Nonameter,
        LineLength::Decameter,
    ];
    for length in &lengths {
        assert!(!length.ai().is_empty());
    }

    let other_meters = [
        OtherMeter::ClimbingRhyme,
        OtherMeter::FallingRhyme,
        OtherMeter::MixedMeter,
        OtherMeter::FreeVerse,
        OtherMeter::BlankVerse,
    ];
    for other_meter in &other_meters {
        assert!(!other_meter.ai().is_empty());
    }
}

#[test]
fn test_meter_equality() {
    let meter1 = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Iamb)
            .length(LineLength::Pentameter)
            .build(),
    );
    let meter2 = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Iamb)
            .length(LineLength::Pentameter)
            .build(),
    );
    let meter3 = Meter::Other(OtherMeter::FreeVerse);

    assert_eq!(meter1, meter2);
    assert_ne!(meter1, meter3);
}

#[test]
fn test_lyrical_meter_equality() {
    let lyrical_meter1 = LyricalMeter::builder()
        .foot(MetricalFoot::Trochee)
        .length(LineLength::Tetrameter)
        .build();
    let lyrical_meter2 = LyricalMeter::builder()
        .foot(MetricalFoot::Trochee)
        .length(LineLength::Tetrameter)
        .build();
    let lyrical_meter3 = LyricalMeter::builder()
        .foot(MetricalFoot::Anapest)
        .length(LineLength::Trimeter)
        .build();

    assert_eq!(lyrical_meter1, lyrical_meter2);
    assert_ne!(lyrical_meter1, lyrical_meter3);
}

#[test]
fn test_meter_builder_defaults() {
    let meter = MeterBuilder::default().build();
    if let Meter::Standard(ref lyrical_meter) = meter {
        assert_eq!(lyrical_meter.foot(), &MetricalFoot::Iamb);
        assert_eq!(lyrical_meter.length(), None);
    } else {
        panic!("Expected Meter::Standard variant");
    }
}

#[test]
fn test_builder_chain_methods() {
    let lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Anapest)
        .length(LineLength::Trimeter)
        .build();

    assert_eq!(lyrical_meter.foot(), &MetricalFoot::Anapest);
    assert_eq!(lyrical_meter.length(), Some(&LineLength::Trimeter));
}
