crate::ix!();

// China Regions
#[derive(OsmPbfFileDownloader,Serialize,Deserialize,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum ChinaRegion {

    #[geofabrik(china="anhui-latest.osm.pbf")]
    #[strum(serialize = "Anhui")] 
    Anhui,

    #[geofabrik(china="beijing-latest.osm.pbf")]
    #[strum(serialize = "Beijing")] 
    Beijing,

    #[geofabrik(china="chongqing-latest.osm.pbf")]
    #[strum(serialize = "Chongqing")] 
    Chongqing,

    #[geofabrik(china="fujian-latest.osm.pbf")]
    #[strum(serialize = "Fujian")] 
    Fujian,

    #[geofabrik(china="gansu-latest.osm.pbf")]
    #[strum(serialize = "Gansu")] 
    Gansu,

    #[geofabrik(china="guangdong-latest.osm.pbf")]
    #[strum(serialize = "Guangdong")] 
    Guangdong,

    #[geofabrik(china="guangxi-latest.osm.pbf")]
    #[strum(serialize = "Guangxi")] 
    Guangxi,

    #[geofabrik(china="guizhou-latest.osm.pbf")]
    #[strum(serialize = "Guizhou")] 
    Guizhou,

    #[geofabrik(china="hainan-latest.osm.pbf")]
    #[strum(serialize = "Hainan")] 
    Hainan,

    #[geofabrik(china="hebei-latest.osm.pbf")]
    #[strum(serialize = "Hebei")] 
    Hebei,

    #[geofabrik(china="heilongjiang-latest.osm.pbf")]
    #[strum(serialize = "Heilongjiang")] 
    Heilongjiang,

    #[geofabrik(china="henan-latest.osm.pbf")]
    #[strum(serialize = "Henan")] 
    Henan,

    #[geofabrik(china="hong-kong-latest.osm.pbf")]
    #[strum(serialize = "Hong Kong")] 
    HongKong,

    #[geofabrik(china="hubei-latest.osm.pbf")]
    #[strum(serialize = "Hubei")] 
    Hubei,

    #[geofabrik(china="hunan-latest.osm.pbf")]
    #[strum(serialize = "Hunan")] 
    Hunan,

    #[geofabrik(china="inner-mongolia-latest.osm.pbf")]
    #[strum(serialize = "Inner Mongolia")] 
    InnerMongolia,

    #[geofabrik(china="jiangsu-latest.osm.pbf")]
    #[strum(serialize = "Jiangsu")] 
    Jiangsu,

    #[geofabrik(china="jiangxi-latest.osm.pbf")]
    #[strum(serialize = "Jiangxi")] 
    Jiangxi,

    #[geofabrik(china="jilin-latest.osm.pbf")]
    #[strum(serialize = "Jilin")] 
    Jilin,

    #[geofabrik(china="liaoning-latest.osm.pbf")]
    #[strum(serialize = "Liaoning")] 
    Liaoning,

    #[geofabrik(china="macau-latest.osm.pbf")]
    #[strum(serialize = "Macau")] 
    Macau,

    #[geofabrik(china="ningxia-latest.osm.pbf")]
    #[strum(serialize = "Ningxia")] 
    Ningxia,

    #[geofabrik(china="qinghai-latest.osm.pbf")]
    #[strum(serialize = "Qinghai")] 
    Qinghai,

    #[geofabrik(china="shaanxi-latest.osm.pbf")]
    #[strum(serialize = "Shaanxi")] 
    Shaanxi,

    #[geofabrik(china="shandong-latest.osm.pbf")]
    #[strum(serialize = "Shandong")] 
    Shandong,

    #[geofabrik(china="shanghai-latest.osm.pbf")]
    #[strum(serialize = "Shanghai")] 
    Shanghai,

    #[geofabrik(china="shanxi-latest.osm.pbf")]
    #[strum(serialize = "Shanxi")] 
    Shanxi,

    #[geofabrik(china="sichuan-latest.osm.pbfj")]
    #[strum(serialize = "Sichuan")] 
    Sichuan,

    #[geofabrik(china="tianjin-latest.osm.pbf")]
    #[strum(serialize = "Tianjin")] 
    Tianjin,

    #[geofabrik(china="tibet-latest.osm.pbf")]
    #[strum(serialize = "Tibet")] 
    Tibet,

    #[default]
    #[geofabrik(china="xinjiang-latest.osm.pbf")]
    #[strum(serialize = "Xinjiang")] 
    Xinjiang,

    #[geofabrik(china="yunnan-latest.osm.pbf")]
    #[strum(serialize = "Yunnan")] 
    Yunnan,

    #[geofabrik(china="zhejiang-latest.osm.pbf")]
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
