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
            AsiaRegion::China(_)                => "CN",
            AsiaRegion::EastTimor               => "TL",
            AsiaRegion::GccStates               => "GCC",
            AsiaRegion::India(_)                => "IN",
            AsiaRegion::Indonesia(_)            => "ID",
            AsiaRegion::Iran                    => "IR",
            AsiaRegion::Iraq                    => "IQ",
            AsiaRegion::IsraelAndPalestine      => "IL-PS",
            AsiaRegion::Japan(_)                => "JP",
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
            AsiaRegion::RussianFederation(_)    => "RU", // same as Europe
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
