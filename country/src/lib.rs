use std::fmt;
use std::str::FromStr;
use serde::{Serialize,Deserialize};

/// The macro that defines and maps countries and codes.
macro_rules! countries {
    (
        $( ($country:ident, $alpha2:ident, $alpha3:ident), )*
    ) => {
        #[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Copy,Clone,Serialize,Deserialize)]
        pub enum Country {
            $($country),*
        }

        #[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Copy,Clone,Serialize,Deserialize)]
        pub enum Iso3166Alpha2 {
            $($alpha2),*
        }

        #[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Copy,Clone,Serialize,Deserialize)]
        pub enum Iso3166Alpha3 {
            $($alpha3),*
        }

        #[derive(Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Copy,Clone,Serialize,Deserialize)]
        pub enum CountryCode {
            Alpha2(Iso3166Alpha2),
            Alpha3(Iso3166Alpha3),
        }

        // From<Country> for Iso3166Alpha2
        impl From<Country> for Iso3166Alpha2 {
            fn from(c: Country) -> Iso3166Alpha2 {
                match c {
                    $(
                        Country::$country => Iso3166Alpha2::$alpha2,
                    )*
                }
            }
        }

        // From<Country> for Iso3166Alpha3
        impl From<Country> for Iso3166Alpha3 {
            fn from(c: Country) -> Iso3166Alpha3 {
                match c {
                    $(
                        Country::$country => Iso3166Alpha3::$alpha3,
                    )*
                }
            }
        }

        // FromStr for Iso3166Alpha2
        impl FromStr for Iso3166Alpha2 {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Iso3166Alpha2, Self::Err> {
                match s {
                    $(stringify!($alpha2) => Ok(Iso3166Alpha2::$alpha2),)*
                    _ => Err("Unknown ISO 3166 Alpha-2 code")
                }
            }
        }

        // FromStr for Iso3166Alpha3
        impl FromStr for Iso3166Alpha3 {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Iso3166Alpha3, Self::Err> {
                match s {
                    $(stringify!($alpha3) => Ok(Iso3166Alpha3::$alpha3),)*
                    _ => Err("Unknown ISO 3166 Alpha-3 code")
                }
            }
        }

        // FromStr for Country
        // This requires that the string matches the variant name exactly.
        // If you want a more human-friendly FromStr, you'll need a lookup table or
        // some logic to convert human-friendly names to variants.
        impl FromStr for Country {
            type Err = &'static str;
            fn from_str(s: &str) -> Result<Country, Self::Err> {
                match s {
                    $(stringify!($country) => Ok(Country::$country),)*
                    _ => Err("Unknown country")
                }
            }
        }

        impl Country {
            pub fn alpha2(&self) -> Iso3166Alpha2 {
                (*self).into()
            }

            pub fn alpha3(&self) -> Iso3166Alpha3 {
                (*self).into()
            }
        }

        // Display for Country (just prints the variant name):
        impl fmt::Display for Country {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self) // Or replace underscores, etc.
            }
        }
    }
}

countries! {
    (Afghanistan                  , AF , AFG), 
    (Albania                      , AL , ALB), 
    (Algeria                      , DZ , DZA), 
    (Andorra                      , AD , AND), 
    (Angola                       , AO , AGO), 
    (AntiguaAndBarbuda            , AG , ATG), 
    (Argentina                    , AR , ARG), 
    (Armenia                      , AM , ARM), 
    (Australia                    , AU , AUS), 
    (Austria                      , AT , AUT), 
    (Azerbaijan                   , AZ , AZE), 
    (Bahamas                      , BS , BHS), 
    (Bahrain                      , BH , BHR), 
    (Bangladesh                   , BD , BGD), 
    (Barbados                     , BB , BRB), 
    (Belarus                      , BY , BLR), 
    (Belgium                      , BE , BEL), 
    (Belize                       , BZ , BLZ), 
    (Benin                        , BJ , BEN), 
    (Bhutan                       , BT , BTN), 
    (Bolivia                      , BO , BOL), 
    (BosniaAndHerzegovina         , BA , BIH), 
    (Botswana                     , BW , BWA), 
    (Brazil                       , BR , BRA), 
    (Brunei                       , BN , BRN), 
    (Bulgaria                     , BG , BGR), 
    (BurkinaFaso                  , BF , BFA), 
    (Burundi                      , BI , BDI), 
    (Cambodia                     , KH , KHM), 
    (Cameroon                     , CM , CMR), 
    (Canada                       , CA , CAN), 
    (CapeVerde                    , CV , CPV), 
    (CentralAfricanRepublic       , CF , CAF), 
    (Chad                         , TD , TCD), 
    (Chile                        , CL , CHL), 
    (China                        , CN , CHN), 
    (Colombia                     , CO , COL), 
    (Comoros                      , KM , COM), 
    (CongoBrazzaville             , CG , COG), // Republic of the Congo
    (CongoKinshasa                , CD , COD), // Democratic Republic of the Congo
    (CostaRica                    , CR , CRI), 
    (Croatia                      , HR , HRV), 
    (Cuba                         , CU , CUB), 
    (Cyprus                       , CY , CYP), 
    (CzechRepublic                , CZ , CZE), 
    (Denmark                      , DK , DNK), 
    (Djibouti                     , DJ , DJI), 
    (Dominica                     , DM , DMA), 
    (DominicanRepublic            , DO , DOM), 
    (Ecuador                      , EC , ECU), 
    (Egypt                        , EG , EGY), 
    (ElSalvador                   , SV , SLV), 
    (EquatorialGuinea             , GQ , GNQ), 
    (Eritrea                      , ER , ERI), 
    (Estonia                      , EE , EST), 
    (Eswatini                     , SZ , SWZ), 
    (Ethiopia                     , ET , ETH), 
    (Fiji                         , FJ , FJI), 
    (Finland                      , FI , FIN), 
    (France                       , FR , FRA), 
    (Gabon                        , GA , GAB), 
    (Gambia                       , GM , GMB), 
    (Georgia                      , GE , GEO), 
    (Germany                      , DE , DEU), 
    (Ghana                        , GH , GHA), 
    (Greece                       , GR , GRC), 
    (Grenada                      , GD , GRD), 
    (Guatemala                    , GT , GTM), 
    (Guinea                       , GN , GIN), 
    (GuineaBissau                 , GW , GNB), 
    (Guyana                       , GY , GUY), 
    (Haiti                        , HT , HTI), 
    (Honduras                     , HN , HND), 
    (Hungary                      , HU , HUN), 
    (Iceland                      , IS , ISL), 
    (India                        , IN , IND), 
    (Indonesia                    , ID , IDN), // Correcting 'Id' -> 'ID'
    (Iran                         , IR , IRN), 
    (Iraq                         , IQ , IRQ), 
    (Ireland                      , IE , IRL), 
    (Israel                       , IL , ISR), 
    (Italy                        , IT , ITA), 
    (IvoryCoast                   , CI , CIV), 
    (Jamaica                      , JM , JAM), 
    (Japan                        , JP , JPN), 
    (Jordan                       , JO , JOR), 
    (Kazakhstan                   , KZ , KAZ), 
    (Kenya                        , KE , KEN), 
    (Kiribati                     , KI , KIR), 
    (Kosovo                       , XK , XKS), // Custom code
    (Kuwait                       , KW , KWT), 
    (Kyrgyzstan                   , KG , KGZ), 
    (Laos                         , LA , LAO), 
    (Latvia                       , LV , LVA), 
    (Lebanon                      , LB , LBN), 
    (Lesotho                      , LS , LSO), 
    (Liberia                      , LR , LBR), 
    (Libya                        , LY , LBY), 
    (Liechtenstein                , LI , LIE), 
    (Lithuania                    , LT , LTU), 
    (Luxembourg                   , LU , LUX), 
    (Madagascar                   , MG , MDG), 
    (Malawi                       , MW , MWI), 
    (Malaysia                     , MY , MYS), 
    (Maldives                     , MV , MDV), 
    (Mali                         , ML , MLI), 
    (Malta                        , MT , MLT), 
    (MarshallIslands              , MH , MHL), 
    (Mauritania                   , MR , MRT), 
    (Mauritius                    , MU , MUS), 
    (Mexico                       , MX , MEX), 
    (Micronesia                   , FM , FSM), 
    (Moldova                      , MD , MDA), 
    (Monaco                       , MC , MCO), 
    (Mongolia                     , MN , MNG), 
    (Montenegro                   , ME , MNE), 
    (Morocco                      , MA , MAR), 
    (Mozambique                   , MZ , MOZ), 
    (Myanmar                      , MM , MMR), 
    (Namibia                      , NA , NAM), 
    (Nauru                        , NR , NRU), 
    (Nepal                        , NP , NPL), 
    (Netherlands                  , NL , NLD), 
    (NewZealand                   , NZ , NZL), 
    (Nicaragua                    , NI , NIC), 
    (Niger                        , NE , NER), 
    (Nigeria                      , NG , NGA), 
    (NorthKorea                   , KP , PRK), 
    (NorthMacedonia               , MK , MKD), 
    (Norway                       , NO , NOR), 
    (Oman                         , OM , OMN), 
    (Pakistan                     , PK , PAK), 
    (Palau                        , PW , PLW), 
    (Palestine                    , PS , PSE), 
    (Panama                       , PA , PAN), 
    (PapuaNewGuinea               , PG , PNG), 
    (Paraguay                     , PY , PRY), 
    (Peru                         , PE , PER), 
    (Philippines                  , PH , PHL), 
    (Poland                       , PL , POL), 
    (Portugal                     , PT , PRT), 
    (Qatar                        , QA , QAT), 
    (Romania                      , RO , ROU), 
    (Russia                       , RU , RUS), 
    (Rwanda                       , RW , RWA), 
    (SaintKittsAndNevis           , KN , KNA), 
    (SaintLucia                   , LC , LCA), 
    (SaintVincentAndTheGrenadines , VC , VCT), 
    (Samoa                        , WS , WSM), 
    (SanMarino                    , SM , SMR), 
    (SaoTomeAndPrincipe           , ST , STP), 
    (SaudiArabia                  , SA , SAU), 
    (Senegal                      , SN , SEN), 
    (Serbia                       , RS , SRB), 
    (Seychelles                   , SC , SYC), 
    (SierraLeone                  , SL , SLE), 
    (Singapore                    , SG , SGP), 
    (Slovakia                     , SK , SVK), 
    (Slovenia                     , SI , SVN), 
    (SolomonIslands               , SB , SLB), 
    (Somalia                      , SO , SOM), 
    (SouthAfrica                  , ZA , ZAF), 
    (SouthKorea                   , KR , KOR), 
    (SouthSudan                   , SS , SSD), 
    (Spain                        , ES , ESP), 
    (SriLanka                     , LK , LKA), 
    (Sudan                        , SD , SDN), 
    (Suriname                     , SR , SUR), 
    (Sweden                       , SE , SWE), 
    (Switzerland                  , CH , CHE), 
    (Syria                        , SY , SYR), 
    (Taiwan                       , TW , TWN), 
    (Tajikistan                   , TJ , TJK), 
    (Tanzania                     , TZ , TZA), 
    (Thailand                     , TH , THA), 
    (TimorLeste                   , TL , TLS), 
    (Togo                         , TG , TGO), 
    (Tonga                        , TO , TON), 
    (TrinidadAndTobago            , TT , TTO), 
    (Tunisia                      , TN , TUN), 
    (Turkey                       , TR , TUR), 
    (Turkmenistan                 , TM , TKM), 
    (Tuvalu                       , TV , TUV), 
    (Uganda                       , UG , UGA), 
    (Ukraine                      , UA , UKR), 
    (UAE                          , AE , ARE), // United Arab Emirates
    (UnitedKingdom                , GB , GBR), // United Kingdom
    (USA                          , US , USA), // United States of America
    (Uruguay                      , UY , URY), 
    (Uzbekistan                   , UZ , UZB), 
    (Vanuatu                      , VU , VUT), 
    (VaticanCity                  , VA , VAT), 
    (Venezuela                    , VE , VEN), 
    (Vietnam                      , VN , VNM), 
    (Yemen                        , YE , YEM), 
    (Zambia                       , ZM , ZMB), 
    (Zimbabwe                     , ZW , ZWE), 
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_country_to_alpha2() {
        // Test a few well-known mappings
        assert_eq!(Country::USA.alpha2(), Iso3166Alpha2::US);
        assert_eq!(Country::UnitedKingdom.alpha2(), Iso3166Alpha2::GB);
        assert_eq!(Country::Germany.alpha2(), Iso3166Alpha2::DE);
        assert_eq!(Country::Japan.alpha2(), Iso3166Alpha2::JP);
        assert_eq!(Country::IvoryCoast.alpha2(), Iso3166Alpha2::CI);
    }

    #[test]
    fn test_country_to_alpha3() {
        assert_eq!(Country::USA.alpha3(), Iso3166Alpha3::USA);
        assert_eq!(Country::UnitedKingdom.alpha3(), Iso3166Alpha3::GBR);
        assert_eq!(Country::Germany.alpha3(), Iso3166Alpha3::DEU);
        assert_eq!(Country::Japan.alpha3(), Iso3166Alpha3::JPN);
        assert_eq!(Country::IvoryCoast.alpha3(), Iso3166Alpha3::CIV);
    }

    #[test]
    fn test_alpha2_from_str() {
        assert_eq!(Iso3166Alpha2::from_str("US").unwrap(), Iso3166Alpha2::US);
        assert_eq!(Iso3166Alpha2::from_str("GB").unwrap(), Iso3166Alpha2::GB);
        assert_eq!(Iso3166Alpha2::from_str("DE").unwrap(), Iso3166Alpha2::DE);
        
        // Test an invalid code
        assert!(Iso3166Alpha2::from_str("XX").is_err());
    }

    #[test]
    fn test_alpha3_from_str() {
        assert_eq!(Iso3166Alpha3::from_str("USA").unwrap(), Iso3166Alpha3::USA);
        assert_eq!(Iso3166Alpha3::from_str("GBR").unwrap(), Iso3166Alpha3::GBR);
        assert_eq!(Iso3166Alpha3::from_str("DEU").unwrap(), Iso3166Alpha3::DEU);

        // Invalid code
        assert!(Iso3166Alpha3::from_str("XXX").is_err());
    }

    #[test]
    fn test_country_from_str() {
        assert_eq!(Country::from_str("USA").unwrap(), Country::USA);
        assert_eq!(Country::from_str("UnitedKingdom").unwrap(), Country::UnitedKingdom);
        assert_eq!(Country::from_str("Germany").unwrap(), Country::Germany);
        assert_eq!(Country::from_str("IvoryCoast").unwrap(), Country::IvoryCoast);

        // Invalid country
        assert!(Country::from_str("UnknownCountry").is_err());
    }

    #[test]
    fn test_country_display() {
        // Just ensures display prints the variant name:
        assert_eq!(format!("{}", Country::USA), "USA");
        assert_eq!(format!("{}", Country::UnitedKingdom), "UnitedKingdom");
        assert_eq!(format!("{}", Country::IvoryCoast), "IvoryCoast");
    }

    #[test]
    fn test_conversions_between_enums() {
        // Check that Country -> Iso3166Alpha2 -> FromStr works round-trip:
        let c = Country::France;
        let a2: Iso3166Alpha2 = c.into();
        assert_eq!(a2, Iso3166Alpha2::FR);
        let a2_str = "FR";
        let parsed_a2 = Iso3166Alpha2::from_str(a2_str).unwrap();
        assert_eq!(parsed_a2, Iso3166Alpha2::FR);

        // Check Country -> Iso3166Alpha3:
        let a3: Iso3166Alpha3 = Country::Brazil.into();
        assert_eq!(a3, Iso3166Alpha3::BRA);
        let a3_str = "BRA";
        let parsed_a3 = Iso3166Alpha3::from_str(a3_str).unwrap();
        assert_eq!(parsed_a3, Iso3166Alpha3::BRA);

        // Ensure alpha2() and alpha3() methods match conversions:
        assert_eq!(Country::Spain.alpha2(), Iso3166Alpha2::ES);
        assert_eq!(Country::Spain.alpha3(), Iso3166Alpha3::ESP);
    }

    #[test]
    fn test_sample_of_difficult_countries() {
        // Test countries with multiple words or special cases:
        assert_eq!(Country::CongoKinshasa.alpha2(), Iso3166Alpha2::CD);
        assert_eq!(Country::CongoKinshasa.alpha3(), Iso3166Alpha3::COD);

        assert_eq!(Country::CongoBrazzaville.alpha2(), Iso3166Alpha2::CG);
        assert_eq!(Country::CongoBrazzaville.alpha3(), Iso3166Alpha3::COG);

        assert_eq!(Country::NorthMacedonia.alpha2(), Iso3166Alpha2::MK);
        assert_eq!(Country::NorthMacedonia.alpha3(), Iso3166Alpha3::MKD);

        // Custom code for Kosovo:
        assert_eq!(Country::Kosovo.alpha2(), Iso3166Alpha2::XK);
        assert_eq!(Country::Kosovo.alpha3(), Iso3166Alpha3::XKS);
    }

    #[test]
    fn test_country_code_enum() {
        // Check CountryCode usage:
        let code = CountryCode::Alpha2(Iso3166Alpha2::US);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::US),
            _ => panic!("Expected Alpha2"),
        }

        let code3 = CountryCode::Alpha3(Iso3166Alpha3::FRA);
        match code3 {
            CountryCode::Alpha3(a3) => assert_eq!(a3, Iso3166Alpha3::FRA),
            _ => panic!("Expected Alpha3"),
        }
    }

    // If you'd like, test serialization/deserialization as well:
    #[test]
    fn test_serde_round_trip() {
        let c = Country::Switzerland;
        let json = serde_json::to_string(&c).expect("Serialize failed");
        let d: Country = serde_json::from_str(&json).expect("Deserialize failed");
        assert_eq!(c, d);

        let a2 = Iso3166Alpha2::JP;
        let json_a2 = serde_json::to_string(&a2).unwrap();
        let d_a2: Iso3166Alpha2 = serde_json::from_str(&json_a2).unwrap();
        assert_eq!(a2, d_a2);
    }
}
