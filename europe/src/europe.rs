crate::ix!();

//--------------------------------------
// Top-level EuropeRegion enum (with all EuropeCountry variants directly under EuropeRegion)
//--------------------------------------
#[derive(FileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum EuropeRegion {

    // Directly included former EuropeCountry variants:
    #[strum(serialize = "Albania")] 
    #[geofabrik(europe="albania-latest.osm.pbf")]
    Albania,

    #[strum(serialize = "Andorra")] 
    #[geofabrik(europe="andorra-latest.osm.pbf")]
    Andorra,

    #[strum(serialize = "Austria")] 
    #[geofabrik(europe="austria-latest.osm.pbf")]
    Austria,

    #[strum(serialize = "Azores")] 
    #[geofabrik(europe="azores-latest.osm.pbf")]
    Azores,

    #[strum(serialize = "Belarus")] 
    #[geofabrik(europe="belarus-latest.osm.pbf")]
    Belarus,

    #[strum(serialize = "Belgium")] 
    #[geofabrik(europe="belgium-latest.osm.pbf")]
    Belgium,

    #[strum(to_string = "Bosnia-Herzegovina", serialize = "Bosnia Herzegovina")] 
    #[geofabrik(europe="bosnia-herzegovina-latest.osm.pbf")]
    BosniaHerzegovina,

    #[strum(serialize = "Bulgaria")] 
    #[geofabrik(europe="bulgaria-latest.osm.pbf")]
    Bulgaria,

    #[strum(serialize = "Croatia")] 
    #[geofabrik(europe="croatia-latest.osm.pbf")]
    Croatia,

    #[strum(serialize = "Cyprus")] 
    #[geofabrik(europe="cyprus-latest.osm.pbf")]
    Cyprus,

    #[strum(serialize = "Czech Republic")] 
    #[geofabrik(europe="czech-republic-latest.osm.pbf")]
    CzechRepublic,

    #[strum(serialize = "Denmark")] 
    #[geofabrik(europe="denmark-latest.osm.pbf")]
    Denmark,

    #[strum(serialize = "Estonia")] 
    #[geofabrik(europe="estonia-latest.osm.pbf")]
    Estonia,

    #[strum(serialize = "Faroe Islands")] 
    #[geofabrik(europe="faroe-islands-latest.osm.pbf")]
    FaroeIslands,

    #[strum(serialize = "Finland")] 
    #[geofabrik(europe="finland-latest.osm.pbf")]
    Finland,

    #[strum(serialize = "Georgia")] 
    #[geofabrik(europe="georgia-latest.osm.pbf")]
    Georgia,

    #[strum(serialize = "Greece")] 
    #[geofabrik(europe="greece-latest.osm.pbf")]
    Greece,

    #[strum(serialize = "Guernsey and Jersey")] 
    #[geofabrik(europe="guernsey-jersey-latest.osm.pbf")]
    GuernseyAndJersey,

    #[strum(serialize = "Hungary")] 
    #[geofabrik(europe="hungary-latest.osm.pbf")]
    Hungary,

    #[strum(serialize = "Iceland")] 
    #[geofabrik(europe="iceland-latest.osm.pbf")]
    Iceland,

    #[strum(serialize = "Ireland and Northern Ireland")] 
    #[geofabrik(europe="ireland-and-northern-ireland-latest.osm.pbf")]
    IrelandAndNorthernIreland,

    #[strum(serialize = "Isle of Man")] 
    #[geofabrik(europe="isle-of-man-latest.osm.pbf")]
    IsleOfMan,

    #[strum(serialize = "Kosovo")] 
    #[geofabrik(europe="kosovo-latest.osm.pbf")]
    Kosovo,

    #[strum(serialize = "Latvia")] 
    #[geofabrik(europe="latvia-latest.osm.pbf")]
    Latvia,

    #[strum(serialize = "Liechtenstein")] 
    #[geofabrik(europe="liechtenstein-latest.osm.pbf")]
    Liechtenstein,

    #[strum(serialize = "Lithuania")] 
    #[geofabrik(europe="lithuania-latest.osm.pbf")]
    Lithuania,

    #[strum(serialize = "Luxembourg")] 
    #[geofabrik(europe="luxembourg-latest.osm.pbf")]
    Luxembourg,

    #[strum(serialize = "Macedonia")] 
    #[geofabrik(europe="macedonia-latest.osm.pbf")]
    Macedonia,

    #[strum(serialize = "Malta")] 
    #[geofabrik(europe="malta-latest.osm.pbf")]
    Malta,

    #[strum(serialize = "Moldova")] 
    #[geofabrik(europe="moldova-latest.osm.pbf")]
    Moldova,

    #[strum(serialize = "Monaco")] 
    #[geofabrik(europe="monaco-latest.osm.pbf")]
    Monaco,

    #[strum(serialize = "Montenegro")] 
    #[geofabrik(europe="montenegro-latest.osm.pbf")]
    Montenegro,

    #[strum(serialize = "Norway")] 
    #[geofabrik(europe="norway-latest.osm.pbf")]
    Norway,

    #[strum(serialize = "Portugal")] 
    #[geofabrik(europe="portugal-latest.osm.pbf")]
    Portugal,

    #[strum(serialize = "Romania")] 
    #[geofabrik(europe="romania-latest.osm.pbf")]
    Romania,

    #[strum(serialize = "Serbia")] 
    #[geofabrik(europe="serbia-latest.osm.pbf")]
    Serbia,

    #[strum(serialize = "Slovakia")] 
    #[geofabrik(europe="slovakia-latest.osm.pbf")]
    Slovakia,

    #[strum(serialize = "Slovenia")] 
    #[geofabrik(europe="slovenia-latest.osm.pbf")]
    Slovenia,

    #[strum(serialize = "Sweden")] 
    #[geofabrik(europe="sweden-latest.osm.pbf")]
    Sweden,

    #[strum(serialize = "Switzerland")] 
    #[geofabrik(europe="switzerland-latest.osm.pbf")]
    Switzerland,

    #[strum(serialize = "Turkey")] 
    #[geofabrik(europe="turkey-latest.osm.pbf")]
    Turkey,

    #[strum(serialize = "Ukraine (with Crimea)", serialize = "Ukraine", serialize = "Ukraine with Crimea")] 
    #[geofabrik(europe="ukraine-latest.osm.pbf")]
    UkraineWithCrimea,

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
        assert_eq!(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(EnglandRegion::GreaterLondon)).abbreviation(), "GB-LND");
        assert_eq!(EuropeRegion::France(FranceRegion::Bretagne).abbreviation(), "FR-E");
    }

    #[test]
    fn test_all_regions() {
        let all = EuropeRegion::all_regions();
        assert!(!all.is_empty(), "Should return a list of all regions");
        // You might check that all known variants appear,
        // or that the count matches an expected value.
    }
}
