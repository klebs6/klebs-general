crate::ix!();

impl TryFrom<AfricaRegion> for Country {
    type Error = AfricaRegionConversionError;

    fn try_from(value: AfricaRegion) -> Result<Self, Self::Error> {
        match value {
            AfricaRegion::Algeria                               => Ok(Country::Algeria),
            AfricaRegion::Angola                                => Ok(Country::Angola),
            AfricaRegion::Benin                                 => Ok(Country::Benin),
            AfricaRegion::Botswana                              => Ok(Country::Botswana),
            AfricaRegion::BurkinaFaso                           => Ok(Country::BurkinaFaso),
            AfricaRegion::Burundi                               => Ok(Country::Burundi),
            AfricaRegion::Cameroon                              => Ok(Country::Cameroon),
            AfricaRegion::CanaryIslands                         => Err(AfricaRegionConversionError::UnsupportedRegion { region: AfricaRegion::CanaryIslands }),
            AfricaRegion::CapeVerde                             => Ok(Country::CapeVerde),
            AfricaRegion::CentralAfricanRepublic                => Ok(Country::CentralAfricanRepublic),
            AfricaRegion::Chad                                  => Ok(Country::Chad),
            AfricaRegion::Comores                               => Ok(Country::Comoros),
            AfricaRegion::CongoRepublicBrazzaville              => Ok(Country::CongoBrazzaville),
            AfricaRegion::CongoDemocraticRepublicKinshasa       => Ok(Country::CongoKinshasa),
            AfricaRegion::Djibouti                              => Ok(Country::Djibouti),
            AfricaRegion::Egypt                                 => Ok(Country::Egypt),
            AfricaRegion::EquatorialGuinea                      => Ok(Country::EquatorialGuinea),
            AfricaRegion::Eritrea                               => Ok(Country::Eritrea),
            AfricaRegion::Ethiopia                              => Ok(Country::Ethiopia),
            AfricaRegion::Gabon                                 => Ok(Country::Gabon),
            AfricaRegion::Ghana                                 => Ok(Country::Ghana),
            AfricaRegion::Guinea                                => Ok(Country::Guinea),
            AfricaRegion::GuineaBissau                          => Ok(Country::GuineaBissau),
            AfricaRegion::IvoryCoast                            => Ok(Country::IvoryCoast),
            AfricaRegion::Kenya                                 => Ok(Country::Kenya),
            AfricaRegion::Lesotho                               => Ok(Country::Lesotho),
            AfricaRegion::Liberia                               => Ok(Country::Liberia),
            AfricaRegion::Libya                                 => Ok(Country::Libya),
            AfricaRegion::Madagascar                            => Ok(Country::Madagascar),
            AfricaRegion::Malawi                                => Ok(Country::Malawi),
            AfricaRegion::Mali                                  => Ok(Country::Mali),
            AfricaRegion::Mauritania                            => Ok(Country::Mauritania),
            AfricaRegion::Mauritius                             => Ok(Country::Mauritius),
            AfricaRegion::Morocco                               => Ok(Country::Morocco),
            AfricaRegion::Mozambique                            => Ok(Country::Mozambique),
            AfricaRegion::Namibia                               => Ok(Country::Namibia),
            AfricaRegion::Niger                                 => Ok(Country::Niger),
            AfricaRegion::Nigeria                               => Ok(Country::Nigeria),
            AfricaRegion::Rwanda                                => Ok(Country::Rwanda),
            AfricaRegion::SaintHelenaAscensionTristanDaCunha    => Err(AfricaRegionConversionError::UnsupportedRegion { region: AfricaRegion::SaintHelenaAscensionTristanDaCunha }),
            AfricaRegion::SaoTomeAndPrincipe                    => Ok(Country::SaoTomeAndPrincipe),
            AfricaRegion::SenegalAndGambia                      => {
                // Combined region. Choose Senegal by convention
                Ok(Country::Senegal)
            },
            AfricaRegion::Seychelles                            => Ok(Country::Seychelles),
            AfricaRegion::SierraLeone                           => Ok(Country::SierraLeone),
            AfricaRegion::Somalia                               => Ok(Country::Somalia),
            AfricaRegion::SouthAfrica                           => Ok(Country::SouthAfrica),
            AfricaRegion::SouthSudan                            => Ok(Country::SouthSudan),
            AfricaRegion::Sudan                                 => Ok(Country::Sudan),
            AfricaRegion::Swaziland                             => Ok(Country::Eswatini), // Eswatini in modern naming
            AfricaRegion::Tanzania                              => Ok(Country::Tanzania),
            AfricaRegion::Togo                                  => Ok(Country::Togo),
            AfricaRegion::Tunisia                               => Ok(Country::Tunisia),
            AfricaRegion::Uganda                                => Ok(Country::Uganda),
            AfricaRegion::Zambia                                => Ok(Country::Zambia),
            AfricaRegion::Zimbabwe                              => Ok(Country::Zimbabwe),
        }
    }
}

impl TryFrom<Country> for AfricaRegion {
    type Error = AfricaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Algeria                => Ok(AfricaRegion::Algeria),
            Country::Angola                 => Ok(AfricaRegion::Angola),
            Country::Benin                  => Ok(AfricaRegion::Benin),
            Country::Botswana               => Ok(AfricaRegion::Botswana),
            Country::BurkinaFaso            => Ok(AfricaRegion::BurkinaFaso),
            Country::Burundi                => Ok(AfricaRegion::Burundi),
            Country::Cameroon               => Ok(AfricaRegion::Cameroon),
            Country::CapeVerde              => Ok(AfricaRegion::CapeVerde),
            Country::CentralAfricanRepublic => Ok(AfricaRegion::CentralAfricanRepublic),
            Country::Chad                   => Ok(AfricaRegion::Chad),
            Country::Comoros                => Ok(AfricaRegion::Comores),
            Country::CongoBrazzaville       => Ok(AfricaRegion::CongoRepublicBrazzaville),
            Country::CongoKinshasa          => Ok(AfricaRegion::CongoDemocraticRepublicKinshasa),
            Country::Djibouti               => Ok(AfricaRegion::Djibouti),
            Country::Egypt                  => Ok(AfricaRegion::Egypt),
            Country::EquatorialGuinea       => Ok(AfricaRegion::EquatorialGuinea),
            Country::Eritrea                => Ok(AfricaRegion::Eritrea),
            Country::Ethiopia               => Ok(AfricaRegion::Ethiopia),
            Country::Gabon                  => Ok(AfricaRegion::Gabon),
            Country::Ghana                  => Ok(AfricaRegion::Ghana),
            Country::Guinea                 => Ok(AfricaRegion::Guinea),
            Country::GuineaBissau           => Ok(AfricaRegion::GuineaBissau),
            Country::IvoryCoast             => Ok(AfricaRegion::IvoryCoast),
            Country::Kenya                  => Ok(AfricaRegion::Kenya),
            Country::Lesotho                => Ok(AfricaRegion::Lesotho),
            Country::Liberia                => Ok(AfricaRegion::Liberia),
            Country::Libya                  => Ok(AfricaRegion::Libya),
            Country::Madagascar             => Ok(AfricaRegion::Madagascar),
            Country::Malawi                 => Ok(AfricaRegion::Malawi),
            Country::Mali                   => Ok(AfricaRegion::Mali),
            Country::Mauritania             => Ok(AfricaRegion::Mauritania),
            Country::Mauritius              => Ok(AfricaRegion::Mauritius),
            Country::Morocco                => Ok(AfricaRegion::Morocco),
            Country::Mozambique             => Ok(AfricaRegion::Mozambique),
            Country::Namibia                => Ok(AfricaRegion::Namibia),
            Country::Niger                  => Ok(AfricaRegion::Niger),
            Country::Nigeria                => Ok(AfricaRegion::Nigeria),
            Country::Rwanda                 => Ok(AfricaRegion::Rwanda),
            Country::SaoTomeAndPrincipe     => Ok(AfricaRegion::SaoTomeAndPrincipe),
            Country::Senegal                => Ok(AfricaRegion::SenegalAndGambia),
            Country::Gambia                 => Ok(AfricaRegion::SenegalAndGambia),
            Country::Seychelles             => Ok(AfricaRegion::Seychelles),
            Country::SierraLeone            => Ok(AfricaRegion::SierraLeone),
            Country::Somalia                => Ok(AfricaRegion::Somalia),
            Country::SouthAfrica            => Ok(AfricaRegion::SouthAfrica),
            Country::SouthSudan             => Ok(AfricaRegion::SouthSudan),
            Country::Sudan                  => Ok(AfricaRegion::Sudan),
            Country::Eswatini               => Ok(AfricaRegion::Swaziland),
            Country::Tanzania               => Ok(AfricaRegion::Tanzania),
            Country::Togo                   => Ok(AfricaRegion::Togo),
            Country::Tunisia                => Ok(AfricaRegion::Tunisia),
            Country::Uganda                 => Ok(AfricaRegion::Uganda),
            Country::Zambia                 => Ok(AfricaRegion::Zambia),
            Country::Zimbabwe               => Ok(AfricaRegion::Zimbabwe),

            // Countries not in Africa or not represented:
            other => Err(AfricaRegionConversionError::NotAfrican { country: other }),
        }
    }
}

//-------------------------------------------------------------
// ISO Code conversions
//-------------------------------------------------------------
impl TryFrom<AfricaRegion> for Iso3166Alpha2 {
    type Error = AfricaRegionConversionError;
    fn try_from(value: AfricaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<AfricaRegion> for Iso3166Alpha3 {
    type Error = AfricaRegionConversionError;
    fn try_from(value: AfricaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<AfricaRegion> for CountryCode {
    type Error = AfricaRegionConversionError;
    fn try_from(value: AfricaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}
