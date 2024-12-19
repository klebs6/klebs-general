crate::ix!();

//--------------------------------------
// Top-level EuropeRegion enum (with all EuropeCountry variants directly under EuropeRegion)
//--------------------------------------
#[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum EuropeRegion {

    // Directly included former EuropeCountry variants:
    #[strum( serialize = "Albania"                                                                         )] Albania,
    #[strum( serialize = "Andorra"                                                                         )] Andorra,
    #[strum( serialize = "Austria"                                                                         )] Austria,
    #[strum( serialize = "Azores"                                                                          )] Azores,
    #[strum( serialize = "Belarus"                                                                         )] Belarus,
    #[strum( serialize = "Belgium"                                                                         )] Belgium,
    #[strum( serialize = "Bosnia-Herzegovina", serialize = "Bosnia Herzegovina"                            )] BosniaHerzegovina,
    #[strum( serialize = "Bulgaria"                                                                        )] Bulgaria,
    #[strum( serialize = "Croatia"                                                                         )] Croatia,
    #[strum( serialize = "Cyprus"                                                                          )] Cyprus,
    #[strum( serialize = "Czech Republic"                                                                  )] CzechRepublic,
    #[strum( serialize = "Denmark"                                                                         )] Denmark,
    #[strum( serialize = "Estonia"                                                                         )] Estonia,
    #[strum( serialize = "Faroe Islands"                                                                   )] FaroeIslands,
    #[strum( serialize = "Finland"                                                                         )] Finland,
    #[strum( serialize = "Georgia"                                                                         )] Georgia,
    #[strum( serialize = "Greece"                                                                          )] Greece,
    #[strum( serialize = "Guernsey and Jersey"                                                             )] GuernseyAndJersey,
    #[strum( serialize = "Hungary"                                                                         )] Hungary,
    #[strum( serialize = "Iceland"                                                                         )] Iceland,
    #[strum( serialize = "Ireland and Northern Ireland"                                                    )] IrelandAndNorthernIreland,
    #[strum( serialize = "Isle of Man"                                                                     )] IsleOfMan,
    #[strum( serialize = "Kosovo"                                                                          )] Kosovo,
    #[strum( serialize = "Latvia"                                                                          )] Latvia,
    #[strum( serialize = "Liechtenstein"                                                                   )] Liechtenstein,
    #[strum( serialize = "Lithuania"                                                                       )] Lithuania,
    #[strum( serialize = "Luxembourg"                                                                      )] Luxembourg,
    #[strum( serialize = "Macedonia"                                                                       )] Macedonia,
    #[strum( serialize = "Malta"                                                                           )] Malta,
    #[strum( serialize = "Moldova"                                                                         )] Moldova,
    #[strum( serialize = "Monaco"                                                                          )] Monaco,
    #[strum( serialize = "Montenegro"                                                                      )] Montenegro,
    #[strum( serialize = "Norway"                                                                          )] Norway,
    #[strum( serialize = "Portugal"                                                                        )] Portugal,
    #[strum( serialize = "Romania"                                                                         )] Romania,
    #[strum( serialize = "Serbia"                                                                          )] Serbia,
    #[strum( serialize = "Slovakia"                                                                        )] Slovakia,
    #[strum( serialize = "Slovenia"                                                                        )] Slovenia,
    #[strum( serialize = "Sweden"                                                                          )] Sweden,
    #[strum( serialize = "Switzerland"                                                                     )] Switzerland,
    #[strum( serialize = "Turkey"                                                                          )] Turkey,
    #[strum( serialize = "Ukraine (with Crimea)", serialize = "Ukraine", serialize = "Ukraine with Crimea" )] UkraineWithCrimea,

    // Subdivided countries:
    France(FranceRegion),
    Germany(GermanyRegion),
    Italy(ItalyRegion),
    Netherlands(NetherlandsRegion),
    Poland(PolandRegion),
    RussianFederation(RussianFederationRegion),
    Spain(SpainRegion),
    UnitedKingdom(UnitedKingdomRegion),
}

impl Default for EuropeRegion {
    fn default() -> Self {
        Self::France(FranceRegion::default())
    }
}

impl EuropeRegion {
    pub fn all_regions() -> Vec<EuropeRegion> {
        let mut v = Vec::new();

        // Add all single-country variants (formerly EuropeCountry variants)
        v.push(EuropeRegion::Albania);
        v.push(EuropeRegion::Andorra);
        v.push(EuropeRegion::Austria);
        v.push(EuropeRegion::Azores);
        v.push(EuropeRegion::Belarus);
        v.push(EuropeRegion::Belgium);
        v.push(EuropeRegion::BosniaHerzegovina);
        v.push(EuropeRegion::Bulgaria);
        v.push(EuropeRegion::Croatia);
        v.push(EuropeRegion::Cyprus);
        v.push(EuropeRegion::CzechRepublic);
        v.push(EuropeRegion::Denmark);
        v.push(EuropeRegion::Estonia);
        v.push(EuropeRegion::FaroeIslands);
        v.push(EuropeRegion::Finland);

        for fr in FranceRegion::iter() {
            v.push(EuropeRegion::France(fr));
        }

        v.push(EuropeRegion::Georgia);

        for gr in GermanyRegion::iter() {
            v.push(EuropeRegion::Germany(gr));
        }

        v.push(EuropeRegion::Greece);
        v.push(EuropeRegion::GuernseyAndJersey);
        v.push(EuropeRegion::Hungary);
        v.push(EuropeRegion::Iceland);
        v.push(EuropeRegion::IrelandAndNorthernIreland);
        v.push(EuropeRegion::IsleOfMan);

        for ir in ItalyRegion::iter() {
            v.push(EuropeRegion::Italy(ir));
        }

        v.push(EuropeRegion::Kosovo);
        v.push(EuropeRegion::Latvia);
        v.push(EuropeRegion::Liechtenstein);
        v.push(EuropeRegion::Lithuania);
        v.push(EuropeRegion::Luxembourg);
        v.push(EuropeRegion::Macedonia);
        v.push(EuropeRegion::Malta);
        v.push(EuropeRegion::Moldova);
        v.push(EuropeRegion::Monaco);
        v.push(EuropeRegion::Montenegro);

        for nr in NetherlandsRegion::iter() {
            v.push(EuropeRegion::Netherlands(nr));
        }

        v.push(EuropeRegion::Norway);

        for pr in PolandRegion::iter() {
            v.push(EuropeRegion::Poland(pr));
        }

        v.push(EuropeRegion::Portugal);
        v.push(EuropeRegion::Romania);

        for rr in RussianFederationRegion::iter() {
            v.push(EuropeRegion::RussianFederation(rr));
        }

        v.push(EuropeRegion::Serbia);
        v.push(EuropeRegion::Slovakia);
        v.push(EuropeRegion::Slovenia);

        for sr in SpainRegion::iter() {
            v.push(EuropeRegion::Spain(sr));
        }

        v.push(EuropeRegion::Sweden);
        v.push(EuropeRegion::Switzerland);
        v.push(EuropeRegion::Turkey);
        v.push(EuropeRegion::UkraineWithCrimea);

        //------------------------------------[united-kingdom]
        // England subdivisions
        for er in EnglandRegion::iter() {
            v.push(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(er)));
        }

        for ukr in UnitedKingdomRegion::iter() {
            match ukr {
                UnitedKingdomRegion::England(_) => { }
                _ => v.push(EuropeRegion::UnitedKingdom(ukr)),
            }
        }

        v
    }

    pub fn name(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod test_europe_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be France(IleDeFrance)
        let def = EuropeRegion::default();
        if let EuropeRegion::France(fr) = def {
            assert_eq!(fr, FranceRegion::IleDeFrance);
        } else {
            panic!("Default EuropeRegion is not France(IleDeFrance)!");
        }
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test a few representative regions
        let regions = vec![
            EuropeRegion::Albania,
            EuropeRegion::France(FranceRegion::Bretagne),
            EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(EnglandRegion::Devon)),
            EuropeRegion::RussianFederation(RussianFederationRegion::SiberianFederalDistrict),
        ];

        for region in regions {
            let serialized = serde_json::to_string(&region).expect("Should serialize");
            let deserialized: EuropeRegion = serde_json::from_str(&serialized)
                .expect("Should deserialize");
            assert_eq!(region, deserialized, "Round trip should preserve the value");
        }
    }

    #[test]
    fn test_from_str() {
        // Check parsing a known variant
        let parsed = EuropeRegion::from_str("Albania").expect("Should parse Albania");
        assert_eq!(parsed, EuropeRegion::Albania);
    }

    #[test]
    fn test_unknown_variant() {
        // Attempt to parse something invalid
        let result = serde_json::from_str::<EuropeRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail to deserialize");
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(EuropeRegion::Albania.abbreviation(), "AL");
        assert_eq!(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(EnglandRegion::GreaterLondon)).abbreviation(), "GB");
        assert_eq!(EuropeRegion::France(FranceRegion::Bretagne).abbreviation(), "FR");
    }

    #[test]
    fn test_all_regions() {
        let all = EuropeRegion::all_regions();
        assert!(!all.is_empty(), "Should return a list of all regions");
        // You might check that all known variants appear,
        // or that the count matches an expected value.
    }
}
