crate::ix!();

//--------------------------------------
// Top-level EuropeRegion enum (with all EuropeCountry variants directly under EuropeRegion)
//--------------------------------------
#[derive(FileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum EuropeRegion {

    // Directly included former EuropeCountry variants:
    #[strum(serialize = "Albania")] 
    #[download_link("https://download.geofabrik.de/europe/albania-latest.osm.pbf")]
    Albania,

    #[strum(serialize = "Andorra")] 
    #[download_link("https://download.geofabrik.de/europe/andorra-latest.osm.pbf")]
    Andorra,

    #[strum(serialize = "Austria")] 
    #[download_link("https://download.geofabrik.de/europe/austria-latest.osm.pbf")]
    Austria,

    #[strum(serialize = "Azores")] 
    #[download_link("https://download.geofabrik.de/europe/azores-latest.osm.pbf")]
    Azores,

    #[strum(serialize = "Belarus")] 
    #[download_link("https://download.geofabrik.de/europe/belarus-latest.osm.pbf")]
    Belarus,

    #[strum(serialize = "Belgium")] 
    #[download_link("https://download.geofabrik.de/europe/belgium-latest.osm.pbf")]
    Belgium,

    #[strum(to_string = "Bosnia-Herzegovina", serialize = "Bosnia Herzegovina")] 
    #[download_link("https://download.geofabrik.de/europe/bosnia-herzegovina-latest.osm.pbf")]
    BosniaHerzegovina,

    #[strum(serialize = "Bulgaria")] 
    #[download_link("https://download.geofabrik.de/europe/bulgaria-latest.osm.pbf")]
    Bulgaria,

    #[strum(serialize = "Croatia")] 
    #[download_link("https://download.geofabrik.de/europe/croatia-latest.osm.pbf")]
    Croatia,

    #[strum(serialize = "Cyprus")] 
    #[download_link("https://download.geofabrik.de/europe/cyprus-latest.osm.pbf")]
    Cyprus,

    #[strum(serialize = "Czech Republic")] 
    #[download_link("https://download.geofabrik.de/europe/czech-republic-latest.osm.pbf")]
    CzechRepublic,

    #[strum(serialize = "Denmark")] 
    #[download_link("https://download.geofabrik.de/europe/denmark-latest.osm.pbf")]
    Denmark,

    #[strum(serialize = "Estonia")] 
    #[download_link("https://download.geofabrik.de/europe/estonia-latest.osm.pbf")]
    Estonia,

    #[strum(serialize = "Faroe Islands")] 
    #[download_link("https://download.geofabrik.de/europe/faroe-islands-latest.osm.pbf")]
    FaroeIslands,

    #[strum(serialize = "Finland")] 
    #[download_link("https://download.geofabrik.de/europe/finland-latest.osm.pbf")]
    Finland,

    #[strum(serialize = "Georgia")] 
    #[download_link("https://download.geofabrik.de/europe/georgia-latest.osm.pbf")]
    Georgia,

    #[strum(serialize = "Greece")] 
    #[download_link("https://download.geofabrik.de/europe/greece-latest.osm.pbf")]
    Greece,

    #[strum(serialize = "Guernsey and Jersey")] 
    #[download_link("https://download.geofabrik.de/europe/guernsey-jersey-latest.osm.pbf")]
    GuernseyAndJersey,

    #[strum(serialize = "Hungary")] 
    #[download_link("https://download.geofabrik.de/europe/hungary-latest.osm.pbf")]
    Hungary,

    #[strum(serialize = "Iceland")] 
    #[download_link("https://download.geofabrik.de/europe/iceland-latest.osm.pbf")]
    Iceland,

    #[strum(serialize = "Ireland and Northern Ireland")] 
    #[download_link("https://download.geofabrik.de/europe/ireland-and-northern-ireland-latest.osm.pbf")]
    IrelandAndNorthernIreland,

    #[strum(serialize = "Isle of Man")] 
    #[download_link("https://download.geofabrik.de/europe/isle-of-man-latest.osm.pbf")]
    IsleOfMan,

    #[strum(serialize = "Kosovo")] 
    #[download_link("https://download.geofabrik.de/europe/kosovo-latest.osm.pbf")]
    Kosovo,

    #[strum(serialize = "Latvia")] 
    #[download_link("https://download.geofabrik.de/europe/latvia-latest.osm.pbf")]
    Latvia,

    #[strum(serialize = "Liechtenstein")] 
    #[download_link("https://download.geofabrik.de/europe/liechtenstein-latest.osm.pbf")]
    Liechtenstein,

    #[strum(serialize = "Lithuania")] 
    #[download_link("https://download.geofabrik.de/europe/lithuania-latest.osm.pbf")]
    Lithuania,

    #[strum(serialize = "Luxembourg")] 
    #[download_link("https://download.geofabrik.de/europe/luxembourg-latest.osm.pbf")]
    Luxembourg,

    #[strum(serialize = "Macedonia")] 
    #[download_link("https://download.geofabrik.de/europe/macedonia-latest.osm.pbf")]
    Macedonia,

    #[strum(serialize = "Malta")] 
    #[download_link("https://download.geofabrik.de/europe/malta-latest.osm.pbf")]
    Malta,

    #[strum(serialize = "Moldova")] 
    #[download_link("https://download.geofabrik.de/europe/moldova-latest.osm.pbf")]
    Moldova,

    #[strum(serialize = "Monaco")] 
    #[download_link("https://download.geofabrik.de/europe/monaco-latest.osm.pbf")]
    Monaco,

    #[strum(serialize = "Montenegro")] 
    #[download_link("https://download.geofabrik.de/europe/montenegro-latest.osm.pbf")]
    Montenegro,

    #[strum(serialize = "Norway")] 
    #[download_link("https://download.geofabrik.de/europe/norway-latest.osm.pbf")]
    Norway,

    #[strum(serialize = "Portugal")] 
    #[download_link("https://download.geofabrik.de/europe/portugal-latest.osm.pbf")]
    Portugal,

    #[strum(serialize = "Romania")] 
    #[download_link("https://download.geofabrik.de/europe/romania-latest.osm.pbf")]
    Romania,

    #[strum(serialize = "Serbia")] 
    #[download_link("https://download.geofabrik.de/europe/serbia-latest.osm.pbf")]
    Serbia,

    #[strum(serialize = "Slovakia")] 
    #[download_link("https://download.geofabrik.de/europe/slovakia-latest.osm.pbf")]
    Slovakia,

    #[strum(serialize = "Slovenia")] 
    #[download_link("https://download.geofabrik.de/europe/slovenia-latest.osm.pbf")]
    Slovenia,

    #[strum(serialize = "Sweden")] 
    #[download_link("https://download.geofabrik.de/europe/sweden-latest.osm.pbf")]
    Sweden,

    #[strum(serialize = "Switzerland")] 
    #[download_link("https://download.geofabrik.de/europe/switzerland-latest.osm.pbf")]
    Switzerland,

    #[strum(serialize = "Turkey")] 
    #[download_link("https://download.geofabrik.de/europe/turkey-latest.osm.pbf")]
    Turkey,

    #[strum(serialize = "Ukraine (with Crimea)", serialize = "Ukraine", serialize = "Ukraine with Crimea")] 
    #[download_link("https://download.geofabrik.de/europe/ukraine-latest.osm.pbf")]
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
