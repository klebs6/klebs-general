crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for AsiaRegion
//-------------------------------------------------------------
impl Abbreviation for AsiaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            AsiaRegion::Afghanistan             => "AF",
            AsiaRegion::Armenia                 => "AM",
            AsiaRegion::Azerbaijan              => "AZ",
            AsiaRegion::Bangladesh              => "BD",
            AsiaRegion::Bhutan                  => "BT",
            AsiaRegion::Cambodia                => "KH",
            AsiaRegion::China(x)                => x.abbreviation(),
            AsiaRegion::EastTimor               => "TL",
            AsiaRegion::GccStates               => "GCC",
            AsiaRegion::India(x)                => x.abbreviation(),
            AsiaRegion::Indonesia(x)            => x.abbreviation(),
            AsiaRegion::Iran                    => "IR",
            AsiaRegion::Iraq                    => "IQ",
            AsiaRegion::IsraelAndPalestine      => "IL-PS",
            AsiaRegion::Japan(x)                => x.abbreviation(),
            AsiaRegion::Jordan                  => "JO",
            AsiaRegion::Kazakhstan              => "KZ",
            AsiaRegion::Kyrgyzstan              => "KG",
            AsiaRegion::Laos                    => "LA",
            AsiaRegion::Lebanon                 => "LB",
            AsiaRegion::MalaysiaSingaporeBrunei => "MY-SG-BN",
            AsiaRegion::Maldives                => "MV",
            AsiaRegion::Mongolia                => "MN",
            AsiaRegion::Myanmar                 => "MM",
            AsiaRegion::Nepal                   => "NP",
            AsiaRegion::NorthKorea              => "KP",
            AsiaRegion::Pakistan                => "PK",
            AsiaRegion::Philippines             => "PH",
            AsiaRegion::RussianFederation(x)    => x.abbreviation(), // same as Europe
            AsiaRegion::SouthKorea              => "KR",
            AsiaRegion::SriLanka                => "LK",
            AsiaRegion::Syria                   => "SY",
            AsiaRegion::Taiwan                  => "TW",
            AsiaRegion::Tajikistan              => "TJ",
            AsiaRegion::Thailand                => "TH",
            AsiaRegion::Turkmenistan            => "TM",
            AsiaRegion::Uzbekistan              => "UZ",
            AsiaRegion::Vietnam                 => "VN",
            AsiaRegion::Yemen                   => "YE",
        }
    }
}

impl TryFromAbbreviation for AsiaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "AF" => return Ok(AsiaRegion::Afghanistan),
            "AM" => return Ok(AsiaRegion::Armenia),
            "AZ" => return Ok(AsiaRegion::Azerbaijan),
            "BD" => return Ok(AsiaRegion::Bangladesh),
            "BT" => return Ok(AsiaRegion::Bhutan),
            "KH" => return Ok(AsiaRegion::Cambodia),
            "TL" => return Ok(AsiaRegion::EastTimor),
            "GCC" => return Ok(AsiaRegion::GccStates),
            "IR" => return Ok(AsiaRegion::Iran),
            "IQ" => return Ok(AsiaRegion::Iraq),
            "IL-PS" => return Ok(AsiaRegion::IsraelAndPalestine),
            "JO" => return Ok(AsiaRegion::Jordan),
            "KZ" => return Ok(AsiaRegion::Kazakhstan),
            "KG" => return Ok(AsiaRegion::Kyrgyzstan),
            "LA" => return Ok(AsiaRegion::Laos),
            "LB" => return Ok(AsiaRegion::Lebanon),
            "MY-SG-BN" => return Ok(AsiaRegion::MalaysiaSingaporeBrunei),
            "MV" => return Ok(AsiaRegion::Maldives),
            "MN" => return Ok(AsiaRegion::Mongolia),
            "MM" => return Ok(AsiaRegion::Myanmar),
            "NP" => return Ok(AsiaRegion::Nepal),
            "KP" => return Ok(AsiaRegion::NorthKorea),
            "PK" => return Ok(AsiaRegion::Pakistan),
            "PH" => return Ok(AsiaRegion::Philippines),
            "KR" => return Ok(AsiaRegion::SouthKorea),
            "LK" => return Ok(AsiaRegion::SriLanka),
            "SY" => return Ok(AsiaRegion::Syria),
            "TW" => return Ok(AsiaRegion::Taiwan),
            "TJ" => return Ok(AsiaRegion::Tajikistan),
            "TH" => return Ok(AsiaRegion::Thailand),
            "TM" => return Ok(AsiaRegion::Turkmenistan),
            "UZ" => return Ok(AsiaRegion::Uzbekistan),
            "VN" => return Ok(AsiaRegion::Vietnam),
            "YE" => return Ok(AsiaRegion::Yemen),
            _ => { /* check subdivided (China, India, Russia, etc.) */ }
        }

        // China subdivision:
        if let Ok(ch) = ChinaRegion::try_from_abbreviation(abbr) {
            return Ok(AsiaRegion::China(ch));
        }
        // India subdivision:
        if let Ok(ind) = IndiaRegion::try_from_abbreviation(abbr) {
            return Ok(AsiaRegion::India(ind));
        }
        // Indonesia subdivision:
        if let Ok(indo) = IndonesiaRegion::try_from_abbreviation(abbr) {
            return Ok(AsiaRegion::Indonesia(indo));
        }
        // Japan subdivision:
        if let Ok(jp) = JapanRegion::try_from_abbreviation(abbr) {
            return Ok(AsiaRegion::Japan(jp));
        }
        // Russian Federation (same struct as in Europe):
        if let Ok(ru) = RussianFederationRegion::try_from_abbreviation(abbr) {
            return Ok(AsiaRegion::RussianFederation(ru));
        }

        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}

///////////////////////////////////////////////////////////////////////////////
// CHINA (Provinces / SARs), ISO 3166‑2:CN
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for ChinaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            ChinaRegion::Anhui         => "CN-AH",
            ChinaRegion::Beijing       => "CN-BJ",
            ChinaRegion::Chongqing     => "CN-CQ",
            ChinaRegion::Fujian        => "CN-FJ",
            ChinaRegion::Gansu         => "CN-GS",
            ChinaRegion::Guangdong     => "CN-GD",
            ChinaRegion::Guangxi       => "CN-GX",
            ChinaRegion::Guizhou       => "CN-GZ",
            ChinaRegion::Hainan        => "CN-HI",
            ChinaRegion::Hebei         => "CN-HE",
            ChinaRegion::Heilongjiang  => "CN-HL",
            ChinaRegion::Henan         => "CN-HA",
            ChinaRegion::HongKong      => "CN-HK",
            ChinaRegion::Hubei         => "CN-HB",
            ChinaRegion::Hunan         => "CN-HN",
            ChinaRegion::InnerMongolia => "CN-NM",
            ChinaRegion::Jiangsu       => "CN-JS",
            ChinaRegion::Jiangxi       => "CN-JX",
            ChinaRegion::Jilin         => "CN-JL",
            ChinaRegion::Liaoning      => "CN-LN",
            ChinaRegion::Macau         => "CN-MO",
            ChinaRegion::Ningxia       => "CN-NX",
            ChinaRegion::Qinghai       => "CN-QH",
            ChinaRegion::Shaanxi       => "CN-SN",
            ChinaRegion::Shandong      => "CN-SD",
            ChinaRegion::Shanghai      => "CN-SH",
            ChinaRegion::Shanxi        => "CN-SX",
            ChinaRegion::Sichuan       => "CN-SC",
            ChinaRegion::Tianjin       => "CN-TJ",
            ChinaRegion::Tibet         => "CN-XZ",
            ChinaRegion::Xinjiang      => "CN-XJ",
            ChinaRegion::Yunnan        => "CN-YN",
            ChinaRegion::Zhejiang      => "CN-ZJ",
        }
    }
}

impl TryFromAbbreviation for ChinaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "CN-AH" => Ok(ChinaRegion::Anhui),
            "CN-BJ" => Ok(ChinaRegion::Beijing),
            "CN-CQ" => Ok(ChinaRegion::Chongqing),
            "CN-FJ" => Ok(ChinaRegion::Fujian),
            "CN-GS" => Ok(ChinaRegion::Gansu),
            "CN-GD" => Ok(ChinaRegion::Guangdong),
            "CN-GX" => Ok(ChinaRegion::Guangxi),
            "CN-GZ" => Ok(ChinaRegion::Guizhou),
            "CN-HI" => Ok(ChinaRegion::Hainan),
            "CN-HE" => Ok(ChinaRegion::Hebei),
            "CN-HL" => Ok(ChinaRegion::Heilongjiang),
            "CN-HA" => Ok(ChinaRegion::Henan),
            "CN-HK" => Ok(ChinaRegion::HongKong),
            "CN-HB" => Ok(ChinaRegion::Hubei),
            "CN-HN" => Ok(ChinaRegion::Hunan),
            "CN-NM" => Ok(ChinaRegion::InnerMongolia),
            "CN-JS" => Ok(ChinaRegion::Jiangsu),
            "CN-JX" => Ok(ChinaRegion::Jiangxi),
            "CN-JL" => Ok(ChinaRegion::Jilin),
            "CN-LN" => Ok(ChinaRegion::Liaoning),
            "CN-MO" => Ok(ChinaRegion::Macau),
            "CN-NX" => Ok(ChinaRegion::Ningxia),
            "CN-QH" => Ok(ChinaRegion::Qinghai),
            "CN-SN" => Ok(ChinaRegion::Shaanxi),
            "CN-SD" => Ok(ChinaRegion::Shandong),
            "CN-SH" => Ok(ChinaRegion::Shanghai),
            "CN-SX" => Ok(ChinaRegion::Shanxi),
            "CN-SC" => Ok(ChinaRegion::Sichuan),
            "CN-TJ" => Ok(ChinaRegion::Tianjin),
            "CN-XZ" => Ok(ChinaRegion::Tibet),
            "CN-XJ" => Ok(ChinaRegion::Xinjiang),
            "CN-YN" => Ok(ChinaRegion::Yunnan),
            "CN-ZJ" => Ok(ChinaRegion::Zhejiang),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// INDIA (Macro Zones, no official ISO)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for IndiaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            IndiaRegion::CentralZone     => "IN-CZ",
            IndiaRegion::EasternZone     => "IN-EZ",
            IndiaRegion::NorthEasternZone => "IN-NEZ",
            IndiaRegion::NorthernZone    => "IN-NZ",
            IndiaRegion::SouthernZone    => "IN-SZ",
            IndiaRegion::WesternZone     => "IN-WZ",
        }
    }
}

impl TryFromAbbreviation for IndiaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "IN-CZ"  => Ok(IndiaRegion::CentralZone),
            "IN-EZ"  => Ok(IndiaRegion::EasternZone),
            "IN-NEZ" => Ok(IndiaRegion::NorthEasternZone),
            "IN-NZ"  => Ok(IndiaRegion::NorthernZone),
            "IN-SZ"  => Ok(IndiaRegion::SouthernZone),
            "IN-WZ"  => Ok(IndiaRegion::WesternZone),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// INDONESIA (Large Islands/Groups, no official ISO macros)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for IndonesiaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            IndonesiaRegion::Java         => "ID-JAVA",
            IndonesiaRegion::Kalimantan   => "ID-KAL",
            IndonesiaRegion::Maluku       => "ID-MAL",
            IndonesiaRegion::NusaTenggara => "ID-NT",
            IndonesiaRegion::Papua        => "ID-PAP",
            IndonesiaRegion::Sulawesi     => "ID-SUL",
            IndonesiaRegion::Sumatra      => "ID-SUM",
        }
    }
}

impl TryFromAbbreviation for IndonesiaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "ID-JAVA" => Ok(IndonesiaRegion::Java),
            "ID-KAL"  => Ok(IndonesiaRegion::Kalimantan),
            "ID-MAL"  => Ok(IndonesiaRegion::Maluku),
            "ID-NT"   => Ok(IndonesiaRegion::NusaTenggara),
            "ID-PAP"  => Ok(IndonesiaRegion::Papua),
            "ID-SUL"  => Ok(IndonesiaRegion::Sulawesi),
            "ID-SUM"  => Ok(IndonesiaRegion::Sumatra),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// JAPAN (8 Macro Regions, custom codes)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for JapanRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            JapanRegion::Chubu   => "JP-CBU",
            JapanRegion::Chugoku => "JP-CGK",
            JapanRegion::Hokkaido => "JP-HKD",
            JapanRegion::Kansai  => "JP-KNS",
            JapanRegion::Kanto   => "JP-KNT",
            JapanRegion::Kyushu  => "JP-KYS",
            JapanRegion::Shikoku => "JP-SHK",
            JapanRegion::Tohoku  => "JP-THK",
        }
    }
}

impl TryFromAbbreviation for JapanRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "JP-CBU" => Ok(JapanRegion::Chubu),
            "JP-CGK" => Ok(JapanRegion::Chugoku),
            "JP-HKD" => Ok(JapanRegion::Hokkaido),
            "JP-KNS" => Ok(JapanRegion::Kansai),
            "JP-KNT" => Ok(JapanRegion::Kanto),
            "JP-KYS" => Ok(JapanRegion::Kyushu),
            "JP-SHK" => Ok(JapanRegion::Shikoku),
            "JP-THK" => Ok(JapanRegion::Tohoku),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}
