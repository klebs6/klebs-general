crate::ix!();

// China Regions
#[derive(Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum ChinaRegion {
    #[strum(serialize = "Anhui"          )] Anhui,
    #[strum(serialize = "Beijing"        )] Beijing,
    #[strum(serialize = "Chongqing"      )] Chongqing,
    #[strum(serialize = "Fujian"         )] Fujian,
    #[strum(serialize = "Gansu"          )] Gansu,
    #[strum(serialize = "Guangdong"      )] Guangdong,
    #[strum(serialize = "Guangxi"        )] Guangxi,
    #[strum(serialize = "Guizhou"        )] Guizhou,
    #[strum(serialize = "Hainan"         )] Hainan,
    #[strum(serialize = "Hebei"          )] Hebei,
    #[strum(serialize = "Heilongjiang"   )] Heilongjiang,
    #[strum(serialize = "Henan"          )] Henan,
    #[strum(serialize = "Hong Kong"      )] HongKong,
    #[strum(serialize = "Hubei"          )] Hubei,
    #[strum(serialize = "Hunan"          )] Hunan,
    #[strum(serialize = "Inner Mongolia" )] InnerMongolia,
    #[strum(serialize = "Jiangsu"        )] Jiangsu,
    #[strum(serialize = "Jiangxi"        )] Jiangxi,
    #[strum(serialize = "Jilin"          )] Jilin,
    #[strum(serialize = "Liaoning"       )] Liaoning,
    #[strum(serialize = "Macau"          )] Macau,
    #[strum(serialize = "Ningxia"        )] Ningxia,
    #[strum(serialize = "Qinghai"        )] Qinghai,
    #[strum(serialize = "Shaanxi"        )] Shaanxi,
    #[strum(serialize = "Shandong"       )] Shandong,
    #[strum(serialize = "Shanghai"       )] Shanghai,
    #[strum(serialize = "Shanxi"         )] Shanxi,
    #[strum(serialize = "Sichuan"        )] Sichuan,
    #[strum(serialize = "Tianjin"        )] Tianjin,
    #[strum(serialize = "Tibet"          )] Tibet,
    #[default]
    #[strum(serialize = "Xinjiang"       )] Xinjiang,
    #[strum(serialize = "Yunnan"         )] Yunnan,
    #[strum(serialize = "Zhejiang"       )] Zhejiang,
}

#[cfg(test)]
mod test_china_region {
    use super::*;
    use serde_json;
    use std::str::FromStr;

    #[test]
    fn test_default() {
        // Default should be Xinjiang
        assert_eq!(ChinaRegion::default(), ChinaRegion::Xinjiang);
    }

    #[test]
    fn test_from_str() {
        let beijing = ChinaRegion::from_str("Beijing").expect("Should parse Beijing");
        assert_eq!(beijing, ChinaRegion::Beijing);
    }

    #[test]
    fn test_round_trip_serialization() {
        let serialized = serde_json::to_string(&ChinaRegion::Guangdong).expect("Serialize");
        let deserialized: ChinaRegion = serde_json::from_str(&serialized).expect("Deserialize");
        assert_eq!(ChinaRegion::Guangdong, deserialized);
    }

    #[test]
    fn test_unknown_variant() {
        let result = serde_json::from_str::<ChinaRegion>("\"Atlantis\"");
        assert!(result.is_err(), "Unknown variant should fail");
    }
}
