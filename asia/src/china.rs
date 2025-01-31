crate::ix!();

// China Regions
#[derive(FileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum ChinaRegion {

    #[download_link("https://download.geofabrik.de/asia/china/anhui-latest.osm.pbf")]
    #[strum(serialize = "Anhui")] 
    Anhui,

    #[download_link("https://download.geofabrik.de/asia/china/beijing-latest.osm.pbf")]
    #[strum(serialize = "Beijing")] 
    Beijing,

    #[download_link("https://download.geofabrik.de/asia/china/chongqing-latest.osm.pbf")]
    #[strum(serialize = "Chongqing")] 
    Chongqing,

    #[download_link("https://download.geofabrik.de/asia/china/fujian-latest.osm.pbf")]
    #[strum(serialize = "Fujian")] 
    Fujian,

    #[download_link("https://download.geofabrik.de/asia/china/gansu-latest.osm.pbf")]
    #[strum(serialize = "Gansu")] 
    Gansu,

    #[download_link("https://download.geofabrik.de/asia/china/guangdong-latest.osm.pbf")]
    #[strum(serialize = "Guangdong")] 
    Guangdong,

    #[download_link("https://download.geofabrik.de/asia/china/guangxi-latest.osm.pbf")]
    #[strum(serialize = "Guangxi")] 
    Guangxi,

    #[download_link("https://download.geofabrik.de/asia/china/guizhou-latest.osm.pbf")]
    #[strum(serialize = "Guizhou")] 
    Guizhou,

    #[download_link("https://download.geofabrik.de/asia/china/hainan-latest.osm.pbf")]
    #[strum(serialize = "Hainan")] 
    Hainan,

    #[download_link("https://download.geofabrik.de/asia/china/hebei-latest.osm.pbf")]
    #[strum(serialize = "Hebei")] 
    Hebei,

    #[download_link("https://download.geofabrik.de/asia/china/heilongjiang-latest.osm.pbf")]
    #[strum(serialize = "Heilongjiang")] 
    Heilongjiang,

    #[download_link("https://download.geofabrik.de/asia/china/henan-latest.osm.pbf")]
    #[strum(serialize = "Henan")] 
    Henan,

    #[download_link("https://download.geofabrik.de/asia/china/hong-kong-latest.osm.pbf")]
    #[strum(serialize = "Hong Kong")] 
    HongKong,

    #[download_link("https://download.geofabrik.de/asia/china/hubei-latest.osm.pbf")]
    #[strum(serialize = "Hubei")] 
    Hubei,

    #[download_link("https://download.geofabrik.de/asia/china/hunan-latest.osm.pbf")]
    #[strum(serialize = "Hunan")] 
    Hunan,

    #[download_link("https://download.geofabrik.de/asia/china/inner-mongolia-latest.osm.pbf")]
    #[strum(serialize = "Inner Mongolia")] 
    InnerMongolia,

    #[download_link("https://download.geofabrik.de/asia/china/jiangsu-latest.osm.pbf")]
    #[strum(serialize = "Jiangsu")] 
    Jiangsu,

    #[download_link("https://download.geofabrik.de/asia/china/jiangxi-latest.osm.pbf")]
    #[strum(serialize = "Jiangxi")] 
    Jiangxi,

    #[download_link("https://download.geofabrik.de/asia/china/jilin-latest.osm.pbf")]
    #[strum(serialize = "Jilin")] 
    Jilin,

    #[download_link("https://download.geofabrik.de/asia/china/liaoning-latest.osm.pbf")]
    #[strum(serialize = "Liaoning")] 
    Liaoning,

    #[download_link("https://download.geofabrik.de/asia/china/macau-latest.osm.pbf")]
    #[strum(serialize = "Macau")] 
    Macau,

    #[download_link("https://download.geofabrik.de/asia/china/ningxia-latest.osm.pbf")]
    #[strum(serialize = "Ningxia")] 
    Ningxia,

    #[download_link("https://download.geofabrik.de/asia/china/qinghai-latest.osm.pbf")]
    #[strum(serialize = "Qinghai")] 
    Qinghai,

    #[download_link("https://download.geofabrik.de/asia/china/shaanxi-latest.osm.pbf")]
    #[strum(serialize = "Shaanxi")] 
    Shaanxi,

    #[download_link("https://download.geofabrik.de/asia/china/shandong-latest.osm.pbf")]
    #[strum(serialize = "Shandong")] 
    Shandong,

    #[download_link("https://download.geofabrik.de/asia/china/shanghai-latest.osm.pbf")]
    #[strum(serialize = "Shanghai")] 
    Shanghai,

    #[download_link("https://download.geofabrik.de/asia/china/shanxi-latest.osm.pbf")]
    #[strum(serialize = "Shanxi")] 
    Shanxi,

    #[download_link("https://download.geofabrik.de/asia/china/sichuan-latest.osm.pbfj")]
    #[strum(serialize = "Sichuan")] 
    Sichuan,

    #[download_link("https://download.geofabrik.de/asia/china/tianjin-latest.osm.pbf")]
    #[strum(serialize = "Tianjin")] 
    Tianjin,

    #[download_link("https://download.geofabrik.de/asia/china/tibet-latest.osm.pbf")]
    #[strum(serialize = "Tibet")] 
    Tibet,

    #[default]
    #[download_link("https://download.geofabrik.de/asia/china/xinjiang-latest.osm.pbf")]
    #[strum(serialize = "Xinjiang")] 
    Xinjiang,

    #[download_link("https://download.geofabrik.de/asia/china/yunnan-latest.osm.pbf")]
    #[strum(serialize = "Yunnan")] 
    Yunnan,

    #[download_link("https://download.geofabrik.de/asia/china/zhejiang-latest.osm.pbf")]
    #[strum(serialize = "Zhejiang")] 
    Zhejiang,
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
