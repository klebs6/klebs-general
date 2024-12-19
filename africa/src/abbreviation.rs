crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for AfricaRegion
//-------------------------------------------------------------
impl Abbreviation for AfricaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            AfricaRegion::Algeria                               => "DZ",
            AfricaRegion::Angola                                => "AO",
            AfricaRegion::Benin                                 => "BJ",
            AfricaRegion::Botswana                              => "BW",
            AfricaRegion::BurkinaFaso                           => "BF",
            AfricaRegion::Burundi                               => "BI",
            AfricaRegion::Cameroon                              => "CM",
            AfricaRegion::CanaryIslands                         => "IC", // not ISO official, using a known code for Canary Islands
            AfricaRegion::CapeVerde                             => "CV",
            AfricaRegion::CentralAfricanRepublic                => "CF",
            AfricaRegion::Chad                                  => "TD",
            AfricaRegion::Comores                               => "KM", // Comoros
            AfricaRegion::CongoRepublicBrazzaville              => "CG",
            AfricaRegion::CongoDemocraticRepublicKinshasa       => "CD",
            AfricaRegion::Djibouti                              => "DJ",
            AfricaRegion::Egypt                                 => "EG",
            AfricaRegion::EquatorialGuinea                      => "GQ",
            AfricaRegion::Eritrea                               => "ER",
            AfricaRegion::Ethiopia                              => "ET",
            AfricaRegion::Gabon                                 => "GA",
            AfricaRegion::Ghana                                 => "GH",
            AfricaRegion::Guinea                                => "GN",
            AfricaRegion::GuineaBissau                          => "GW",
            AfricaRegion::IvoryCoast                            => "CI",
            AfricaRegion::Kenya                                 => "KE",
            AfricaRegion::Lesotho                               => "LS",
            AfricaRegion::Liberia                               => "LR",
            AfricaRegion::Libya                                 => "LY",
            AfricaRegion::Madagascar                            => "MG",
            AfricaRegion::Malawi                                => "MW",
            AfricaRegion::Mali                                  => "ML",
            AfricaRegion::Mauritania                            => "MR",
            AfricaRegion::Mauritius                             => "MU",
            AfricaRegion::Morocco                               => "MA",
            AfricaRegion::Mozambique                            => "MZ",
            AfricaRegion::Namibia                               => "NA",
            AfricaRegion::Niger                                 => "NE",
            AfricaRegion::Nigeria                               => "NG",
            AfricaRegion::Rwanda                                => "RW",
            AfricaRegion::SaintHelenaAscensionTristanDaCunha    => "SH-AC-TA", // combined territory, no single country code
            AfricaRegion::SaoTomeAndPrincipe                    => "ST",
            AfricaRegion::SenegalAndGambia                      => "SN-GM", // combined region
            AfricaRegion::Seychelles                            => "SC",
            AfricaRegion::SierraLeone                           => "SL",
            AfricaRegion::Somalia                               => "SO",
            AfricaRegion::SouthAfrica                           => "ZA",
            AfricaRegion::SouthSudan                            => "SS",
            AfricaRegion::Sudan                                 => "SD",
            AfricaRegion::Swaziland                             => "SZ", // Eswatini (SZ)
            AfricaRegion::Tanzania                              => "TZ",
            AfricaRegion::Togo                                  => "TG",
            AfricaRegion::Tunisia                               => "TN",
            AfricaRegion::Uganda                                => "UG",
            AfricaRegion::Zambia                                => "ZM",
            AfricaRegion::Zimbabwe                              => "ZW",
        }
    }
}
