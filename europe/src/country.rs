crate::ix!();

//////////////////////////////////////////////////////////////
// TryFrom<EuropeRegion> for Country
//////////////////////////////////////////////////////////////

impl TryFrom<EuropeRegion> for Country {
    type Error = EuropeRegionConversionError;

    fn try_from(value: EuropeRegion) -> Result<Self, Self::Error> {
        match value {
            // Direct matches for non-subdivided countries:
            EuropeRegion::Albania => Ok(Country::Albania),
            EuropeRegion::Andorra => Ok(Country::Andorra),
            EuropeRegion::Austria => Ok(Country::Austria),
            EuropeRegion::Azores  => {
                // Azores are part of Portugal
                Ok(Country::Portugal)
            },
            EuropeRegion::Belarus           => Ok(Country::Belarus),
            EuropeRegion::Belgium           => Ok(Country::Belgium),
            EuropeRegion::BosniaHerzegovina => Ok(Country::BosniaAndHerzegovina),
            EuropeRegion::Bulgaria          => Ok(Country::Bulgaria),
            EuropeRegion::Croatia           => Ok(Country::Croatia),
            EuropeRegion::Cyprus            => Ok(Country::Cyprus),
            EuropeRegion::CzechRepublic     => Ok(Country::CzechRepublic),
            EuropeRegion::Denmark           => Ok(Country::Denmark),
            EuropeRegion::Estonia           => Ok(Country::Estonia),
            EuropeRegion::FaroeIslands      => {
                // Faroe Islands are part of the Kingdom of Denmark but not represented as a separate country in `country::Country`.
                // If you want to map them to Denmark:
                // Ok(Country::Denmark)
                // Otherwise:
                Err(EuropeRegionConversionError::UnsupportedRegion { region: EuropeRegion::FaroeIslands })
            },
            EuropeRegion::Finland           => Ok(Country::Finland),
            EuropeRegion::France(_)         => Ok(Country::France),
            EuropeRegion::Georgia           => Ok(Country::Georgia),
            EuropeRegion::Germany(_)        => Ok(Country::Germany),
            EuropeRegion::Greece            => Ok(Country::Greece),
            EuropeRegion::GuernseyAndJersey => {
                // Guernsey and Jersey are Crown dependencies of the UK.
                // Not separate countries in `country::Country`.
                // If you want to treat them as UK:
                // Ok(Country::UnitedKingdom)
                // Otherwise:
                Err(EuropeRegionConversionError::UnsupportedRegion { region: EuropeRegion::GuernseyAndJersey })
            },
            EuropeRegion::Hungary                   => Ok(Country::Hungary),
            EuropeRegion::Iceland                   => Ok(Country::Iceland),
            EuropeRegion::IrelandAndNorthernIreland => {
                // This is a combined region. We'll map it to Ireland by convention.
                Ok(Country::Ireland)
            },
            EuropeRegion::IsleOfMan => {
                // Another Crown dependency of the UK.
                // If desired, map to UK:
                // Ok(Country::UnitedKingdom)
                Err(EuropeRegionConversionError::UnsupportedRegion { region: EuropeRegion::IsleOfMan })
            },
            EuropeRegion::Italy(_)      => Ok(Country::Italy),
            EuropeRegion::Kosovo        => Ok(Country::Kosovo),
            EuropeRegion::Latvia        => Ok(Country::Latvia),
            EuropeRegion::Liechtenstein => Ok(Country::Liechtenstein),
            EuropeRegion::Lithuania     => Ok(Country::Lithuania),
            EuropeRegion::Luxembourg    => Ok(Country::Luxembourg),
            EuropeRegion::Macedonia     => {
                // Macedonia maps to NorthMacedonia in `country::Country`.
                Ok(Country::NorthMacedonia)
            },
            EuropeRegion::Malta                => Ok(Country::Malta),
            EuropeRegion::Moldova              => Ok(Country::Moldova),
            EuropeRegion::Monaco               => Ok(Country::Monaco),
            EuropeRegion::Montenegro           => Ok(Country::Montenegro),
            EuropeRegion::Netherlands(_)       => Ok(Country::Netherlands),
            EuropeRegion::Norway               => Ok(Country::Norway),
            EuropeRegion::Poland(_)            => Ok(Country::Poland),
            EuropeRegion::Portugal             => Ok(Country::Portugal),
            EuropeRegion::Romania              => Ok(Country::Romania),
            EuropeRegion::RussianFederation(_) => Ok(Country::Russia),
            EuropeRegion::Serbia               => Ok(Country::Serbia),
            EuropeRegion::Slovakia             => Ok(Country::Slovakia),
            EuropeRegion::Slovenia             => Ok(Country::Slovenia),
            EuropeRegion::Spain(_)             => Ok(Country::Spain),
            EuropeRegion::Sweden               => Ok(Country::Sweden),
            EuropeRegion::Switzerland          => Ok(Country::Switzerland),
            EuropeRegion::Turkey               => Ok(Country::Turkey),
            EuropeRegion::UkraineWithCrimea    => Ok(Country::Ukraine),
            EuropeRegion::UnitedKingdom(_)     => Ok(Country::UnitedKingdom),
        }
    }
}

//////////////////////////////////////////////////////////////
// TryFrom<Country> for EuropeRegion
//////////////////////////////////////////////////////////////

impl TryFrom<Country> for EuropeRegion {
    type Error = EuropeRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Albania              => Ok(EuropeRegion::Albania),
            Country::Andorra              => Ok(EuropeRegion::Andorra),
            Country::Austria              => Ok(EuropeRegion::Austria),
            Country::Belarus              => Ok(EuropeRegion::Belarus),
            Country::Belgium              => Ok(EuropeRegion::Belgium),
            Country::BosniaAndHerzegovina => Ok(EuropeRegion::BosniaHerzegovina),
            Country::Bulgaria             => Ok(EuropeRegion::Bulgaria),
            Country::Croatia              => Ok(EuropeRegion::Croatia),
            Country::Cyprus               => Ok(EuropeRegion::Cyprus),
            Country::CzechRepublic        => Ok(EuropeRegion::CzechRepublic),
            Country::Denmark              => Ok(EuropeRegion::Denmark),
            Country::Estonia              => Ok(EuropeRegion::Estonia),
            Country::Finland              => Ok(EuropeRegion::Finland),
            Country::France               => Ok(EuropeRegion::France(FranceRegion::default())),
            Country::Georgia              => Ok(EuropeRegion::Georgia),
            Country::Germany              => Ok(EuropeRegion::Germany(GermanyRegion::default())),
            Country::Greece               => Ok(EuropeRegion::Greece),
            Country::Hungary              => Ok(EuropeRegion::Hungary),
            Country::Iceland              => Ok(EuropeRegion::Iceland),
            Country::Ireland              => {
                // No direct Ireland-only variant, we have IrelandAndNorthernIreland.
                Ok(EuropeRegion::IrelandAndNorthernIreland)
            },
            Country::Italy          => Ok(EuropeRegion::Italy(ItalyRegion::default())),
            Country::Kosovo         => Ok(EuropeRegion::Kosovo),
            Country::Latvia         => Ok(EuropeRegion::Latvia),
            Country::Liechtenstein  => Ok(EuropeRegion::Liechtenstein),
            Country::Lithuania      => Ok(EuropeRegion::Lithuania),
            Country::Luxembourg     => Ok(EuropeRegion::Luxembourg),
            Country::Malta          => Ok(EuropeRegion::Malta),
            Country::Moldova        => Ok(EuropeRegion::Moldova),
            Country::Monaco         => Ok(EuropeRegion::Monaco),
            Country::Montenegro     => Ok(EuropeRegion::Montenegro),
            Country::Netherlands    => Ok(EuropeRegion::Netherlands(NetherlandsRegion::default())),
            Country::NorthMacedonia => {
                // Maps back to Macedonia variant
                Ok(EuropeRegion::Macedonia)
            },
            Country::Norway        => Ok(EuropeRegion::Norway),
            Country::Poland        => Ok(EuropeRegion::Poland(PolandRegion::default())),
            Country::Portugal      => Ok(EuropeRegion::Portugal),
            Country::Romania       => Ok(EuropeRegion::Romania),
            Country::Russia        => Ok(EuropeRegion::RussianFederation(RussianFederationRegion::default())),
            Country::Serbia        => Ok(EuropeRegion::Serbia),
            Country::Slovakia      => Ok(EuropeRegion::Slovakia),
            Country::Slovenia      => Ok(EuropeRegion::Slovenia),
            Country::Spain         => Ok(EuropeRegion::Spain(SpainRegion::default())),
            Country::Sweden        => Ok(EuropeRegion::Sweden),
            Country::Switzerland   => Ok(EuropeRegion::Switzerland),
            Country::Turkey        => Ok(EuropeRegion::Turkey),
            Country::Ukraine       => Ok(EuropeRegion::UkraineWithCrimea),
            Country::UnitedKingdom => Ok(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::default())),

            // Any country not listed above is not in EuropeRegion:
            other => Err(EuropeRegionConversionError::NotEuropean { country: other }),
        }
    }
}

//////////////////////////////////////////////////////////////
// TryFrom<EuropeRegion> for Iso3166Alpha2, Iso3166Alpha3, and CountryCode
//////////////////////////////////////////////////////////////

impl TryFrom<EuropeRegion> for Iso3166Alpha2 {
    type Error = EuropeRegionConversionError;

    fn try_from(value: EuropeRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?; // Convert EuropeRegion to Country first
        Ok(country.alpha2())
    }
}

impl TryFrom<EuropeRegion> for Iso3166Alpha3 {
    type Error = EuropeRegionConversionError;

    fn try_from(value: EuropeRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<EuropeRegion> for CountryCode {
    type Error = EuropeRegionConversionError;

    fn try_from(value: EuropeRegion) -> Result<Self, Self::Error> {
        // Prefer Alpha2 codes:
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}

// Now you can use `try_into()` and `try_from()` conversions instead of custom traits.
// For example:
// let eu_region = EuropeRegion::France(FranceRegion::Bretagne);
// let c: Country = eu_region.try_into().expect("Should map to France");
// let alpha2: Iso3166Alpha2 = eu_region.try_into().expect("Should map to FR");

#[macro_export] macro_rules! impl_into_country_for_subregions {
    ( $( ($subregion:ty, $country:ident) ),* ) => {
        $(
            impl From<$subregion> for Country {
                fn from(_value: $subregion) -> Self {
                    Country::$country
                }
            }
        )*
    }
}

impl_into_country_for_subregions!{
    (FranceRegion            , France)        ,
    (GermanyRegion           , Germany)       ,
    (ItalyRegion             , Italy)         ,
    (NetherlandsRegion       , Netherlands)   ,
    (PolandRegion            , Poland)        ,
    (RussianFederationRegion , Russia)        ,
    (SpainRegion             , Spain)         ,
    (EnglandRegion           , UnitedKingdom) ,
    (UnitedKingdomRegion     , UnitedKingdom)
}

#[cfg(test)]
mod country_integration_tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn test_europe_region_to_country_success() {
        // Test a representative set of EuropeRegion variants that map directly to Country.
        assert_eq!(Country::try_from(EuropeRegion::Albania).unwrap(), Country::Albania);
        assert_eq!(Country::try_from(EuropeRegion::Austria).unwrap(), Country::Austria);
        assert_eq!(Country::try_from(EuropeRegion::France(FranceRegion::Alsace)).unwrap(), Country::France);
        assert_eq!(Country::try_from(EuropeRegion::Germany(GermanyRegion::Berlin)).unwrap(), Country::Germany);
        assert_eq!(Country::try_from(EuropeRegion::Italy(ItalyRegion::Centro)).unwrap(), Country::Italy);
        assert_eq!(Country::try_from(EuropeRegion::Netherlands(NetherlandsRegion::NoordHolland)).unwrap(), Country::Netherlands);
        assert_eq!(Country::try_from(EuropeRegion::Poland(PolandRegion::WojewodztwoMazowieckie)).unwrap(), Country::Poland);
        assert_eq!(Country::try_from(EuropeRegion::RussianFederation(RussianFederationRegion::CentralFederalDistrict)).unwrap(), Country::Russia);
        assert_eq!(Country::try_from(EuropeRegion::Spain(SpainRegion::Madrid)).unwrap(), Country::Spain);
        assert_eq!(Country::try_from(EuropeRegion::UnitedKingdom(UnitedKingdomRegion::Scotland)).unwrap(), Country::UnitedKingdom);

        // Check a few non-subdivided countries:
        assert_eq!(Country::try_from(EuropeRegion::Norway).unwrap(), Country::Norway);
        assert_eq!(Country::try_from(EuropeRegion::Moldova).unwrap(), Country::Moldova);
        assert_eq!(Country::try_from(EuropeRegion::Romania).unwrap(), Country::Romania);
        assert_eq!(Country::try_from(EuropeRegion::Turkey).unwrap(), Country::Turkey);

        // Check special combined regions:
        assert_eq!(Country::try_from(EuropeRegion::IrelandAndNorthernIreland).unwrap(), Country::Ireland);
        assert_eq!(Country::try_from(EuropeRegion::Macedonia).unwrap(), Country::NorthMacedonia);
        assert_eq!(Country::try_from(EuropeRegion::UkraineWithCrimea).unwrap(), Country::Ukraine);

        // Check Azores -> Portugal mapping:
        assert_eq!(Country::try_from(EuropeRegion::Azores).unwrap(), Country::Portugal);
    }

    #[test]
    fn test_europe_region_to_country_errors() {
        // Test unsupported regions that return errors:
        match Country::try_from(EuropeRegion::FaroeIslands) {
            Err(EuropeRegionConversionError::UnsupportedRegion { region }) => {
                assert_eq!(region, EuropeRegion::FaroeIslands);
            },
            _ => panic!("Expected UnsupportedRegion for FaroeIslands"),
        }

        match Country::try_from(EuropeRegion::GuernseyAndJersey) {
            Err(EuropeRegionConversionError::UnsupportedRegion { region }) => {
                assert_eq!(region, EuropeRegion::GuernseyAndJersey);
            },
            _ => panic!("Expected UnsupportedRegion for Guernsey and Jersey"),
        }

        match Country::try_from(EuropeRegion::IsleOfMan) {
            Err(EuropeRegionConversionError::UnsupportedRegion { region }) => {
                assert_eq!(region, EuropeRegion::IsleOfMan);
            },
            _ => panic!("Expected UnsupportedRegion for Isle of Man"),
        }
    }

    #[test]
    fn test_country_to_europe_region_success() {
        // Test European countries mapping back to default or known EuropeRegion variants:
        assert_eq!(EuropeRegion::try_from(Country::Albania).unwrap(), EuropeRegion::Albania);
        assert_eq!(EuropeRegion::try_from(Country::France).unwrap(), EuropeRegion::France(FranceRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Germany).unwrap(), EuropeRegion::Germany(GermanyRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Italy).unwrap(), EuropeRegion::Italy(ItalyRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Netherlands).unwrap(), EuropeRegion::Netherlands(NetherlandsRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Poland).unwrap(), EuropeRegion::Poland(PolandRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Russia).unwrap(), EuropeRegion::RussianFederation(RussianFederationRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::Spain).unwrap(), EuropeRegion::Spain(SpainRegion::default()));
        assert_eq!(EuropeRegion::try_from(Country::UnitedKingdom).unwrap(), EuropeRegion::UnitedKingdom(UnitedKingdomRegion::default()));

        // Check special combined regions:
        assert_eq!(EuropeRegion::try_from(Country::Ireland).unwrap(), EuropeRegion::IrelandAndNorthernIreland);
        assert_eq!(EuropeRegion::try_from(Country::NorthMacedonia).unwrap(), EuropeRegion::Macedonia);
        assert_eq!(EuropeRegion::try_from(Country::Ukraine).unwrap(), EuropeRegion::UkraineWithCrimea);
    }

    #[test]
    fn test_country_to_europe_region_errors() {
        // Test a non-European country:
        match EuropeRegion::try_from(Country::Brazil) {
            Err(EuropeRegionConversionError::NotEuropean { country }) => {
                assert_eq!(country, Country::Brazil);
            },
            _ => panic!("Expected NotEuropean for Brazil"),
        }

        match EuropeRegion::try_from(Country::USA) {
            Err(EuropeRegionConversionError::NotEuropean { country }) => {
                assert_eq!(country, Country::USA);
            },
            _ => panic!("Expected NotEuropean for USA"),
        }
    }

    #[test]
    fn test_europe_region_to_iso_codes() {
        // Test converting EuropeRegion to Alpha2, Alpha3, and CountryCode:
        let region = EuropeRegion::France(FranceRegion::Bretagne);
        let alpha2: Iso3166Alpha2 = region.try_into().unwrap();
        let alpha3: Iso3166Alpha3 = region.try_into().unwrap();
        let code: CountryCode = region.try_into().unwrap();
        assert_eq!(alpha2, Iso3166Alpha2::FR);
        assert_eq!(alpha3, Iso3166Alpha3::FRA);
        match code {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::FR),
            _ => panic!("Expected Alpha2 code"),
        }

        let region2 = EuropeRegion::Germany(GermanyRegion::Berlin);
        let alpha2_ger: Iso3166Alpha2 = region2.try_into().unwrap();
        let alpha3_ger: Iso3166Alpha3 = region2.try_into().unwrap();
        let code_ger: CountryCode = region2.try_into().unwrap();
        assert_eq!(alpha2_ger, Iso3166Alpha2::DE);
        assert_eq!(alpha3_ger, Iso3166Alpha3::DEU);
        match code_ger {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::DE),
            _ => panic!("Expected Alpha2 code"),
        }

        // Test a non-subdivided region:
        let region3 = EuropeRegion::Hungary;
        let alpha2_hu: Iso3166Alpha2 = region3.try_into().unwrap();
        let alpha3_hu: Iso3166Alpha3 = region3.try_into().unwrap();
        let code_hu: CountryCode = region3.try_into().unwrap();
        assert_eq!(alpha2_hu, Iso3166Alpha2::HU);
        assert_eq!(alpha3_hu, Iso3166Alpha3::HUN);
        match code_hu {
            CountryCode::Alpha2(a2) => assert_eq!(a2, Iso3166Alpha2::HU),
            _ => panic!("Expected Alpha2 code"),
        }
    }

    #[test]
    fn test_subregion_into_country() {
        // Test that each subregion enum maps directly to the correct Country via `From`:
        let fr: Country = FranceRegion::Bretagne.into();
        assert_eq!(fr, Country::France);

        let gr: Country = GermanyRegion::Berlin.into();
        assert_eq!(gr, Country::Germany);

        let it: Country = ItalyRegion::NordEst.into();
        assert_eq!(it, Country::Italy);

        let nl: Country = NetherlandsRegion::NoordHolland.into();
        assert_eq!(nl, Country::Netherlands);

        let pl: Country = PolandRegion::WojewodztwoMazowieckie.into();
        assert_eq!(pl, Country::Poland);

        let ru: Country = RussianFederationRegion::CentralFederalDistrict.into();
        assert_eq!(ru, Country::Russia);

        let es: Country = SpainRegion::Madrid.into();
        assert_eq!(es, Country::Spain);

        let eng: Country = EnglandRegion::Devon.into();
        assert_eq!(eng, Country::UnitedKingdom);

        let ukr: Country = UnitedKingdomRegion::Scotland.into();
        assert_eq!(ukr, Country::UnitedKingdom);
    }

    #[test]
    fn test_round_trip_country_europe_region() {
        // Test going from EuropeRegion -> Country -> EuropeRegion for a few complex cases:
        let region = EuropeRegion::France(FranceRegion::Bretagne);
        let c: Country = region.try_into().unwrap();
        let back: EuropeRegion = c.try_into().unwrap();
        // Bretagne won't be preserved because we defaulted subdivisions, but we should still get France(...) back:
        if let EuropeRegion::France(_) = back {
            // Good, France variant returned
        } else {
            panic!("Expected a France(...) variant");
        }

        let region2 = EuropeRegion::UnitedKingdom(UnitedKingdomRegion::England(EnglandRegion::GreaterLondon));
        let c2: Country = region2.try_into().unwrap();
        let back2: EuropeRegion = c2.try_into().unwrap();
        if let EuropeRegion::UnitedKingdom(_) = back2 {
            // Good, UK variant returned
        } else {
            panic!("Expected a UnitedKingdom(...) variant");
        }

        let region3 = EuropeRegion::Macedonia;
        let c3: Country = region3.try_into().unwrap();
        assert_eq!(c3, Country::NorthMacedonia);
        let back3: EuropeRegion = c3.try_into().unwrap();
        assert_eq!(back3, EuropeRegion::Macedonia);

        let region4 = EuropeRegion::IrelandAndNorthernIreland;
        let c4: Country = region4.try_into().unwrap();
        assert_eq!(c4, Country::Ireland);
        let back4: EuropeRegion = c4.try_into().unwrap();
        // Returns IrelandAndNorthernIreland as per our default logic
        assert_eq!(back4, EuropeRegion::IrelandAndNorthernIreland);
    }

    #[test]
    fn test_error_conditions_iso_codes() {
        // Test converting non-European countries from Europe's perspective to ISO codes should fail:
        // Not directly possible since we start from EuropeRegion, which is always European.
        // Instead, test a region that fails conversion to Country:
        match Iso3166Alpha2::try_from(EuropeRegion::IsleOfMan) {
            Err(EuropeRegionConversionError::UnsupportedRegion { region }) => {
                assert_eq!(region, EuropeRegion::IsleOfMan);
            },
            _ => panic!("Expected UnsupportedRegion for Isle of Man -> Country -> Iso3166Alpha2"),
        }
    }
}
