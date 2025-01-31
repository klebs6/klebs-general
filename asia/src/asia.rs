crate::ix!();

//-------------------------------------------------------------
// AsiaRegion Enum
//-------------------------------------------------------------
#[derive(FileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum AsiaRegion {

    #[download_link("https://download.geofabrik.de/asia/afghanistan-latest.osm.pbf")]
    Afghanistan,

    #[download_link("https://download.geofabrik.de/asia/armenia-latest.osm.pbf")]
    Armenia,

    #[download_link("https://download.geofabrik.de/asia/azerbaijan-latest.osm.pbf")]
    Azerbaijan,

    #[download_link("https://download.geofabrik.de/asia/bangladesh-latest.osm.pbf")]
    Bangladesh,

    #[download_link("https://download.geofabrik.de/asia/bhutan-latest.osm.pbf")]
    Bhutan,

    #[download_link("https://download.geofabrik.de/asia/cambodia-latest.osm.pbf")]
    Cambodia,

    China(ChinaRegion),

    #[download_link("https://download.geofabrik.de/asia/east-timor-latest.osm.pbf")]
    EastTimor,

    #[strum(serialize = "GCC States")]
    #[download_link("https://download.geofabrik.de/asia/gcc-states-latest.osm.pbf")]
    GccStates, // Bahrain, Kuwait, Oman, Qatar, Saudi Arabia, UAE combined

    India(IndiaRegion),

    Indonesia(IndonesiaRegion),

    #[download_link("https://download.geofabrik.de/asia/iran-latest.osm.pbf")]
    Iran,

    #[download_link("https://download.geofabrik.de/asia/iraq-latest.osm.pbf")]
    Iraq,

    #[strum(serialize = "Israel and Palestine")]
    #[download_link("https://download.geofabrik.de/asia/israel-and-palestine-latest.osm.pbf")]
    IsraelAndPalestine,

    Japan(JapanRegion),

    #[download_link("https://download.geofabrik.de/asia/jordan-latest.osm.pbf")]
    Jordan,

    #[download_link("https://download.geofabrik.de/asia/kazakhstan-latest.osm.pbf")]
    Kazakhstan,

    #[download_link("https://download.geofabrik.de/asia/kyrgyzstan-latest.osm.pbf")]
    Kyrgyzstan,

    #[download_link("https://download.geofabrik.de/asia/laos-latest.osm.pbf")]
    Laos,

    #[download_link("https://download.geofabrik.de/asia/lebanon-latest.osm.pbf")]
    Lebanon,

    #[strum(serialize = "Malaysia, Singapore, and Brunei")]
    #[download_link("https://download.geofabrik.de/asia/malaysia-singapore-brunei-latest.osm.pbf")]
    MalaysiaSingaporeBrunei, // Malaysia, Singapore, Brunei combined

    #[download_link("https://download.geofabrik.de/asia/maldives-latest.osm.pbf")]
    Maldives,

    #[download_link("https://download.geofabrik.de/asia/mongolia-latest.osm.pbf")]
    Mongolia,

    #[download_link("https://download.geofabrik.de/asia/myanmar-latest.osm.pbf")]
    Myanmar,

    #[download_link("https://download.geofabrik.de/asia/nepal-latest.osm.pbf")]
    Nepal,

    #[download_link("https://download.geofabrik.de/asia/north-korea-latest.osm.pbf")]
    NorthKorea,

    #[download_link("https://download.geofabrik.de/asia/pakistan-latest.osm.pbf")]
    Pakistan,

    #[download_link("https://download.geofabrik.de/asia/philippines-latest.osm.pbf")]
    Philippines,

    RussianFederation(RussianFederationRegion),

    #[download_link("https://download.geofabrik.de/asia/south-korea-latest.osm.pbf")]
    SouthKorea,

    #[download_link("https://download.geofabrik.de/asia/sri-lanka-latest.osm.pbf")]
    SriLanka,

    #[download_link("https://download.geofabrik.de/asia/syria-latest.osm.pbf")]
    Syria,

    #[download_link("https://download.geofabrik.de/asia/taiwan-latest.osm.pbf")]
    Taiwan,

    #[download_link("https://download.geofabrik.de/asia/tajikistan-latest.osm.pbf")]
    Tajikistan,

    #[download_link("https://download.geofabrik.de/asia/thailand-latest.osm.pbf")]
    Thailand,

    #[download_link("https://download.geofabrik.de/asia/turkmenistan-latest.osm.pbf")]
    Turkmenistan,

    #[download_link("https://download.geofabrik.de/asia/uzbekistan-latest.osm.pbf")]
    Uzbekistan,

    #[download_link("https://download.geofabrik.de/asia/vietnam-latest.osm.pbf")]
    Vietnam,

    #[download_link("https://download.geofabrik.de/asia/yemen-latest.osm.pbf")]
    Yemen,
}

impl Default for AsiaRegion {
    fn default() -> Self {
        // Arbitrarily pick a default: China(Xinjiang)
        AsiaRegion::China(ChinaRegion::default())
    }
}

#[cfg(test)]
mod test_asia_region {
    use super::*;
    use serde_json;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be China(Xinjiang)
        let def = AsiaRegion::default();
        if let AsiaRegion::China(cr) = def {
            assert_eq!(cr, ChinaRegion::Xinjiang);
        } else {
            panic!("Default AsiaRegion is not China(Xinjiang)!");
        }
    }

    #[test]
    fn test_from_str() {
        // Check parsing a known non-subdivided variant
        let parsed = AsiaRegion::from_str("Armenia").expect("Should parse Armenia");
        assert_eq!(parsed, AsiaRegion::Armenia);

        // Check parsing subdivided: since we only store as maps, we should test via serialization
        // For direct from_str for AsiaRegion, we rely on strum, which might only parse top-level variants.
        // Because of the presence of fields, strum might not parse subdivided variants directly from_str.
        // This is expected. We'll handle subdivided variants through serialization tests.
    }

    #[test]
    fn test_abbreviations() {
        assert_eq!(AsiaRegion::Afghanistan.abbreviation(), "AF");
        assert_eq!(AsiaRegion::China(ChinaRegion::Beijing).abbreviation(), "CN");
        assert_eq!(AsiaRegion::India(IndiaRegion::CentralZone).abbreviation(), "IN");
        assert_eq!(AsiaRegion::IsraelAndPalestine.abbreviation(), "IL-PS");
        assert_eq!(AsiaRegion::MalaysiaSingaporeBrunei.abbreviation(), "MY-SG-BN");
        assert_eq!(AsiaRegion::RussianFederation(RussianFederationRegion::CentralFederalDistrict).abbreviation(), "RU");
    }

    #[test]
    fn test_europe_region_variants_listed() {
        // Ensure AsiaRegion variants are accessible:
        let variants = AsiaRegion::VARIANTS;
        assert!(variants.contains(&"Armenia"));
        assert!(variants.contains(&"China"));
        assert!(variants.contains(&"Japan"));
    }

    #[test]
    fn test_asia_region_to_country_success() {
        assert_eq!(Country::try_from(AsiaRegion::Afghanistan).unwrap(), Country::Afghanistan);
        assert_eq!(Country::try_from(AsiaRegion::Iran).unwrap(), Country::Iran);
        assert_eq!(Country::try_from(AsiaRegion::China(ChinaRegion::Hebei)).unwrap(), Country::China);
        assert_eq!(Country::try_from(AsiaRegion::India(IndiaRegion::SouthernZone)).unwrap(), Country::India);
        assert_eq!(Country::try_from(AsiaRegion::Indonesia(IndonesiaRegion::Sumatra)).unwrap(), Country::Indonesia);
        assert_eq!(Country::try_from(AsiaRegion::Japan(JapanRegion::Kanto)).unwrap(), Country::Japan);
        assert_eq!(Country::try_from(AsiaRegion::RussianFederation(RussianFederationRegion::SiberianFederalDistrict)).unwrap(), Country::Russia);
        assert_eq!(Country::try_from(AsiaRegion::Nepal).unwrap(), Country::Nepal);

        // Special combined regions:
        // IsraelAndPalestine -> Israel
        assert_eq!(Country::try_from(AsiaRegion::IsraelAndPalestine).unwrap(), Country::Israel);
        // MalaysiaSingaporeBrunei -> Malaysia
        assert_eq!(Country::try_from(AsiaRegion::MalaysiaSingaporeBrunei).unwrap(), Country::Malaysia);
    }

    #[test]
    fn test_asia_region_to_country_errors() {
        // GCC States is a combined region not directly mapped to a single country
        match Country::try_from(AsiaRegion::GccStates) {
            Err( AsiaRegionConversionError::UnsupportedRegion { .. } ) => {}
            _ => panic!("Expected UnsupportedRegion for GCC States"),
        }
    }

    #[test]
    fn test_country_to_asia_region_success() {
        // Simple direct mappings
        assert_eq!(AsiaRegion::try_from(Country::Afghanistan).unwrap(), AsiaRegion::Afghanistan);
        assert_eq!(AsiaRegion::try_from(Country::Iran).unwrap(), AsiaRegion::Iran);
        assert_eq!(AsiaRegion::try_from(Country::China).unwrap(), AsiaRegion::China(ChinaRegion::default()));
        assert_eq!(AsiaRegion::try_from(Country::India).unwrap(), AsiaRegion::India(IndiaRegion::default()));
        assert_eq!(AsiaRegion::try_from(Country::Indonesia).unwrap(), AsiaRegion::Indonesia(IndonesiaRegion::default()));
        assert_eq!(AsiaRegion::try_from(Country::Japan).unwrap(), AsiaRegion::Japan(JapanRegion::default()));
        assert_eq!(AsiaRegion::try_from(Country::Russia).unwrap(), AsiaRegion::RussianFederation(RussianFederationRegion::default()));

        // Special combined regions:
        // Israel -> IsraelAndPalestine
        assert_eq!(AsiaRegion::try_from(Country::Israel).unwrap(), AsiaRegion::IsraelAndPalestine);
        // Malaysia -> MalaysiaSingaporeBrunei
        assert_eq!(AsiaRegion::try_from(Country::Malaysia).unwrap(), AsiaRegion::MalaysiaSingaporeBrunei);
    }

    #[test]
    fn test_country_to_asia_region_errors() {
        // Test a non-Asian country:
        match AsiaRegion::try_from(Country::Brazil) {
            Err(AsiaRegionConversionError::NotAsian { .. }) => {}
            _ => panic!("Expected NotAsian for Brazil"),
        }

        match AsiaRegion::try_from(Country::USA) {
            Err(AsiaRegionConversionError::NotAsian { .. }) => {}
            _ => panic!("Expected NotAsian for USA"),
        }
    }

    #[test]
    fn test_iso_code_conversions() {
        let region = AsiaRegion::Japan(JapanRegion::Hokkaido);
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();

        assert_eq!(alpha2, Iso3166Alpha2::JP);
        assert_eq!(alpha3, Iso3166Alpha3::JPN);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::JP),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_serialize_deserialize_non_subdivided() {
        // Test round-trip for a non-subdivided country
        let region = AsiaRegion::Laos;
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: AsiaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_subdivided() {
        // Test round-trip for a subdivided country (China)
        let region = AsiaRegion::China(ChinaRegion::Beijing);
        let serialized = serde_json::to_string(&region).expect("Serialize");
        let deserialized: AsiaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(region, deserialized);

        // Test round-trip for a subdivided country (Indonesia)
        let region2 = AsiaRegion::Indonesia(IndonesiaRegion::Sulawesi);
        let serialized2 = serde_json::to_string(&region2).expect("Serialize");
        let deserialized2: AsiaRegion = serde_json::from_str(&serialized2).expect("Deserialize");
        assert_eq!(region2, deserialized2);
    }

    #[test]
    fn test_round_trip_country_asia_region() {
        // Complex region: IsraelAndPalestine -> Israel -> IsraelAndPalestine
        let region = AsiaRegion::IsraelAndPalestine;
        let c: Country = region.try_into().unwrap();
        assert_eq!(c, Country::Israel);
        let back: AsiaRegion = c.try_into().unwrap();
        assert_eq!(back, AsiaRegion::IsraelAndPalestine);

        // Complex region: MalaysiaSingaporeBrunei -> Malaysia -> MalaysiaSingaporeBrunei
        let region2 = AsiaRegion::MalaysiaSingaporeBrunei;
        let c2: Country = region2.try_into().unwrap();
        assert_eq!(c2, Country::Malaysia);
        let back2: AsiaRegion = c2.try_into().unwrap();
        assert_eq!(back2, AsiaRegion::MalaysiaSingaporeBrunei);
    }

    #[test]
    fn test_error_conditions_iso_codes() {
        // Test converting a non-mappable AsiaRegion to Country -> ISO codes:
        // GCC States
        match Iso3166Alpha2::try_from(AsiaRegion::GccStates) {
            Err(AsiaRegionConversionError::UnsupportedRegion { .. }) => {}
            _ => panic!("Expected error for GCC States -> ISO codes"),
        }
    }
}
