crate::ix!();

impl Abbreviation for EuropeRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            EuropeRegion::Albania                   => "AL",
            EuropeRegion::Andorra                   => "AD",
            EuropeRegion::Austria                   => "AT",
            EuropeRegion::Azores                    => "PT-AC", // Azores (Autonomous Region of Portugal)
            EuropeRegion::Belarus                   => "BY",
            EuropeRegion::Belgium                   => "BE",
            EuropeRegion::BosniaHerzegovina         => "BA",
            EuropeRegion::Bulgaria                  => "BG",
            EuropeRegion::Croatia                   => "HR",
            EuropeRegion::Cyprus                    => "CY",
            EuropeRegion::CzechRepublic             => "CZ",
            EuropeRegion::Denmark                   => "DK",
            EuropeRegion::Estonia                   => "EE",
            EuropeRegion::FaroeIslands              => "FO",
            EuropeRegion::Finland                   => "FI",
            EuropeRegion::Georgia                   => "GE",
            EuropeRegion::Greece                    => "GR",
            EuropeRegion::GuernseyAndJersey         => "GG-JE",
            EuropeRegion::Hungary                   => "HU",
            EuropeRegion::Iceland                   => "IS",
            EuropeRegion::IrelandAndNorthernIreland => "IE-GB-NI", // Ireland and Northern Ireland combined is non-standard. Let's just combine codes:
            EuropeRegion::IsleOfMan                 => "IM",
            EuropeRegion::Kosovo                    => "XK",
            EuropeRegion::Latvia                    => "LV",
            EuropeRegion::Liechtenstein             => "LI",
            EuropeRegion::Lithuania                 => "LT",
            EuropeRegion::Luxembourg                => "LU",
            EuropeRegion::Macedonia                 => "MK",
            EuropeRegion::Malta                     => "MT",
            EuropeRegion::Moldova                   => "MD",
            EuropeRegion::Monaco                    => "MC",
            EuropeRegion::Montenegro                => "ME",
            EuropeRegion::Norway                    => "NO",
            EuropeRegion::Portugal                  => "PT",
            EuropeRegion::Romania                   => "RO",
            EuropeRegion::Serbia                    => "RS",
            EuropeRegion::Slovakia                  => "SK",
            EuropeRegion::Slovenia                  => "SI",
            EuropeRegion::Sweden                    => "SE",
            EuropeRegion::Switzerland               => "CH",
            EuropeRegion::Turkey                    => "TR",
            EuropeRegion::UkraineWithCrimea         => "UA-CR", // Ukraine (with Crimea) is non-standard; let's just "UA-CR"

            // Subdivided countries - call their abbreviation methods, which we will fully enumerate below:
            EuropeRegion::France(x)            => x.abbreviation(),
            EuropeRegion::Germany(x)           => x.abbreviation(),
            EuropeRegion::Italy(x)             => x.abbreviation(),
            EuropeRegion::Netherlands(x)       => x.abbreviation(),
            EuropeRegion::Poland(x)            => x.abbreviation(),
            EuropeRegion::RussianFederation(x) => x.abbreviation(),
            EuropeRegion::Spain(x)             => x.abbreviation(),
            EuropeRegion::UnitedKingdom(x)     => x.abbreviation(),
        }
    }
}

impl TryFromAbbreviation for EuropeRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        // Direct matches for single variants:
        match abbr {
            "AL" => return Ok(EuropeRegion::Albania),
            "AD" => return Ok(EuropeRegion::Andorra),
            "AT" => return Ok(EuropeRegion::Austria),
            "PT-AC" => return Ok(EuropeRegion::Azores),
            "BY" => return Ok(EuropeRegion::Belarus),
            "BE" => return Ok(EuropeRegion::Belgium),
            "BA" => return Ok(EuropeRegion::BosniaHerzegovina),
            "BG" => return Ok(EuropeRegion::Bulgaria),
            "HR" => return Ok(EuropeRegion::Croatia),
            "CY" => return Ok(EuropeRegion::Cyprus),
            "CZ" => return Ok(EuropeRegion::CzechRepublic),
            "DK" => return Ok(EuropeRegion::Denmark),
            "EE" => return Ok(EuropeRegion::Estonia),
            "FO" => return Ok(EuropeRegion::FaroeIslands),
            "FI" => return Ok(EuropeRegion::Finland),
            "GE" => return Ok(EuropeRegion::Georgia),
            "GR" => return Ok(EuropeRegion::Greece),
            "GG-JE" => return Ok(EuropeRegion::GuernseyAndJersey),
            "HU" => return Ok(EuropeRegion::Hungary),
            "IS" => return Ok(EuropeRegion::Iceland),
            "IE-GB-NI" => return Ok(EuropeRegion::IrelandAndNorthernIreland),
            "IM" => return Ok(EuropeRegion::IsleOfMan),
            "XK" => return Ok(EuropeRegion::Kosovo),
            "LV" => return Ok(EuropeRegion::Latvia),
            "LI" => return Ok(EuropeRegion::Liechtenstein),
            "LT" => return Ok(EuropeRegion::Lithuania),
            "LU" => return Ok(EuropeRegion::Luxembourg),
            "MK" => return Ok(EuropeRegion::Macedonia),
            "MT" => return Ok(EuropeRegion::Malta),
            "MD" => return Ok(EuropeRegion::Moldova),
            "MC" => return Ok(EuropeRegion::Monaco),
            "ME" => return Ok(EuropeRegion::Montenegro),
            "NO" => return Ok(EuropeRegion::Norway),
            "PT" => return Ok(EuropeRegion::Portugal),
            "RO" => return Ok(EuropeRegion::Romania),
            "RS" => return Ok(EuropeRegion::Serbia),
            "SK" => return Ok(EuropeRegion::Slovakia),
            "SI" => return Ok(EuropeRegion::Slovenia),
            "SE" => return Ok(EuropeRegion::Sweden),
            "CH" => return Ok(EuropeRegion::Switzerland),
            "TR" => return Ok(EuropeRegion::Turkey),
            "UA-CR" => return Ok(EuropeRegion::UkraineWithCrimea),
            _ => { /* check subdivided matches below */ }
        }

        // Subdivided countries:
        if let Ok(fr) = FranceRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::France(fr));
        }
        if let Ok(de) = GermanyRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::Germany(de));
        }
        if let Ok(it) = ItalyRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::Italy(it));
        }
        if let Ok(nl) = NetherlandsRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::Netherlands(nl));
        }
        if let Ok(pl) = PolandRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::Poland(pl));
        }
        if let Ok(ru) = RussiaRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::RussianFederation(ru));
        }
        if let Ok(es) = SpainRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::Spain(es));
        }
        if let Ok(uk) = UnitedKingdomRegion::try_from_abbreviation(abbr) {
            return Ok(EuropeRegion::UnitedKingdom(uk));
        }

        Err(TryFromAbbreviationError::InvalidAbbreviation)
    }
}

///////////////////////////////////////////////////////////////////////////////
// FRANCE (Older 22 Regions, Pre‑2016)
///////////////////////////////////////////////////////////////////////////////

/// Below are official older ISO 3166‑2 region codes for metropolitan + overseas:
impl Abbreviation for FranceRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            FranceRegion::Alsace           => "FR-A",
            FranceRegion::Aquitaine        => "FR-B",
            FranceRegion::Auvergne         => "FR-C",
            FranceRegion::BasseNormandie   => "FR-P",
            FranceRegion::Bourgogne        => "FR-D",
            FranceRegion::Bretagne         => "FR-E",
            FranceRegion::Centre           => "FR-F",
            FranceRegion::ChampagneArdenne => "FR-G",
            FranceRegion::Corse            => "FR-H",
            FranceRegion::FrancheComte     => "FR-I",
            FranceRegion::Guadeloupe       => "FR-GP",
            FranceRegion::Guyane           => "FR-GF",
            FranceRegion::HauteNormandie   => "FR-Q",
            FranceRegion::IleDeFrance      => "FR-J",
            FranceRegion::LanguedocRoussillon => "FR-K",
            FranceRegion::Limousin         => "FR-L",
            FranceRegion::Lorraine         => "FR-M",
            FranceRegion::Martinique       => "FR-MQ",
            FranceRegion::Mayotte          => "FR-YT",
            FranceRegion::MidiPyrenees     => "FR-N",
            FranceRegion::NordPasDeCalais  => "FR-O",
            FranceRegion::PaysDeLaLoire    => "FR-R",
            FranceRegion::Picardie         => "FR-S",
            FranceRegion::PoitouCharentes  => "FR-T",
            FranceRegion::ProvenceAlpesCoteDAzur => "FR-U",
            FranceRegion::Reunion          => "FR-RE",
            FranceRegion::RhoneAlpes       => "FR-V",
        }
    }
}

impl TryFromAbbreviation for FranceRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "FR-A"  => Ok(FranceRegion::Alsace),
            "FR-B"  => Ok(FranceRegion::Aquitaine),
            "FR-C"  => Ok(FranceRegion::Auvergne),
            "FR-P"  => Ok(FranceRegion::BasseNormandie),
            "FR-D"  => Ok(FranceRegion::Bourgogne),
            "FR-E"  => Ok(FranceRegion::Bretagne),
            "FR-F"  => Ok(FranceRegion::Centre),
            "FR-G"  => Ok(FranceRegion::ChampagneArdenne),
            "FR-H"  => Ok(FranceRegion::Corse),
            "FR-I"  => Ok(FranceRegion::FrancheComte),
            "FR-GP" => Ok(FranceRegion::Guadeloupe),
            "FR-GF" => Ok(FranceRegion::Guyane),
            "FR-Q"  => Ok(FranceRegion::HauteNormandie),
            "FR-J"  => Ok(FranceRegion::IleDeFrance),
            "FR-K"  => Ok(FranceRegion::LanguedocRoussillon),
            "FR-L"  => Ok(FranceRegion::Limousin),
            "FR-M"  => Ok(FranceRegion::Lorraine),
            "FR-MQ" => Ok(FranceRegion::Martinique),
            "FR-YT" => Ok(FranceRegion::Mayotte),
            "FR-N"  => Ok(FranceRegion::MidiPyrenees),
            "FR-O"  => Ok(FranceRegion::NordPasDeCalais),
            "FR-R"  => Ok(FranceRegion::PaysDeLaLoire),
            "FR-S"  => Ok(FranceRegion::Picardie),
            "FR-T"  => Ok(FranceRegion::PoitouCharentes),
            "FR-U"  => Ok(FranceRegion::ProvenceAlpesCoteDAzur),
            "FR-RE" => Ok(FranceRegion::Reunion),
            "FR-V"  => Ok(FranceRegion::RhoneAlpes),
            _       => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// GERMANY
///////////////////////////////////////////////////////////////////////////////

/// We use standard ISO codes for single states, except your "mit Berlin" combos
/// are not official. We'll approximate them as "DE-BB+BE," etc.
impl Abbreviation for GermanyRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            GermanyRegion::BadenWurttemberg        => "DE-BW",
            GermanyRegion::Bayern                  => "DE-BY",
            GermanyRegion::Berlin                  => "DE-BE",
            GermanyRegion::BrandenburgMitBerlin    => "DE-BB+BE", // Not standard, but combined
            GermanyRegion::Bremen                  => "DE-HB",
            GermanyRegion::Hamburg                 => "DE-HH",
            GermanyRegion::Hessen                  => "DE-HE",
            GermanyRegion::MecklenburgVorpommern   => "DE-MV",
            GermanyRegion::NiedersachsenMitBremen  => "DE-NI+HB", // Combined
            GermanyRegion::NordrheinWestfalen      => "DE-NW",
            GermanyRegion::RheinlandPfalz          => "DE-RP",
            GermanyRegion::Saarland                => "DE-SL",
            GermanyRegion::Sachsen                 => "DE-SN",
            GermanyRegion::SachsenAnhalt           => "DE-ST",
            GermanyRegion::SchleswigHolstein       => "DE-SH",
            GermanyRegion::Thueringen              => "DE-TH",
        }
    }
}

impl TryFromAbbreviation for GermanyRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "DE-BW"       => Ok(GermanyRegion::BadenWurttemberg),
            "DE-BY"       => Ok(GermanyRegion::Bayern),
            "DE-BE"       => Ok(GermanyRegion::Berlin),
            "DE-BB+BE"    => Ok(GermanyRegion::BrandenburgMitBerlin),
            "DE-HB"       => Ok(GermanyRegion::Bremen),
            "DE-HH"       => Ok(GermanyRegion::Hamburg),
            "DE-HE"       => Ok(GermanyRegion::Hessen),
            "DE-MV"       => Ok(GermanyRegion::MecklenburgVorpommern),
            "DE-NI+HB"    => Ok(GermanyRegion::NiedersachsenMitBremen),
            "DE-NW"       => Ok(GermanyRegion::NordrheinWestfalen),
            "DE-RP"       => Ok(GermanyRegion::RheinlandPfalz),
            "DE-SL"       => Ok(GermanyRegion::Saarland),
            "DE-SN"       => Ok(GermanyRegion::Sachsen),
            "DE-ST"       => Ok(GermanyRegion::SachsenAnhalt),
            "DE-SH"       => Ok(GermanyRegion::SchleswigHolstein),
            "DE-TH"       => Ok(GermanyRegion::Thueringen),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// ITALY (Macroareas, not official ISO)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for ItalyRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            ItalyRegion::Centro   => "IT-CEN",
            ItalyRegion::Isole    => "IT-ISO",
            ItalyRegion::NordEst  => "IT-NE",
            ItalyRegion::NordOvest => "IT-NO",
            ItalyRegion::Sud      => "IT-SUD",
        }
    }
}

impl TryFromAbbreviation for ItalyRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "IT-CEN" => Ok(ItalyRegion::Centro),
            "IT-ISO" => Ok(ItalyRegion::Isole),
            "IT-NE"  => Ok(ItalyRegion::NordEst),
            "IT-NO"  => Ok(ItalyRegion::NordOvest),
            "IT-SUD" => Ok(ItalyRegion::Sud),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// NETHERLANDS (Provinces, official ISO 3166-2:NL codes)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for NetherlandsRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            NetherlandsRegion::Drenthe      => "NL-DR",
            NetherlandsRegion::Flevoland    => "NL-FL",
            NetherlandsRegion::Friesland    => "NL-FR",
            NetherlandsRegion::Gelderland   => "NL-GE",
            NetherlandsRegion::Groningen    => "NL-GR",
            NetherlandsRegion::Limburg      => "NL-LI",
            NetherlandsRegion::NoordBrabant => "NL-NB",
            NetherlandsRegion::NoordHolland => "NL-NH",
            NetherlandsRegion::Overijssel   => "NL-OV",
            NetherlandsRegion::Utrecht      => "NL-UT",
            NetherlandsRegion::Zeeland      => "NL-ZE",
            NetherlandsRegion::ZuidHolland  => "NL-ZH",
        }
    }
}

impl TryFromAbbreviation for NetherlandsRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "NL-DR" => Ok(NetherlandsRegion::Drenthe),
            "NL-FL" => Ok(NetherlandsRegion::Flevoland),
            "NL-FR" => Ok(NetherlandsRegion::Friesland),
            "NL-GE" => Ok(NetherlandsRegion::Gelderland),
            "NL-GR" => Ok(NetherlandsRegion::Groningen),
            "NL-LI" => Ok(NetherlandsRegion::Limburg),
            "NL-NB" => Ok(NetherlandsRegion::NoordBrabant),
            "NL-NH" => Ok(NetherlandsRegion::NoordHolland),
            "NL-OV" => Ok(NetherlandsRegion::Overijssel),
            "NL-UT" => Ok(NetherlandsRegion::Utrecht),
            "NL-ZE" => Ok(NetherlandsRegion::Zeeland),
            "NL-ZH" => Ok(NetherlandsRegion::ZuidHolland),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// POLAND (Voivodeships, official ISO 3166-2:PL)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for PolandRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            PolandRegion::WojewodztwoDolnoslaskie    => "PL-DS",
            PolandRegion::WojewodztwoKujawskoPomorskie => "PL-KP",
            PolandRegion::WojewodztwoLodzkie         => "PL-LD",
            PolandRegion::WojewodztwoLubelskie       => "PL-LU",
            PolandRegion::WojewodztwoLubuskie        => "PL-LB",
            PolandRegion::WojewodztwoMalopolskie     => "PL-MA",
            PolandRegion::WojewodztwoMazowieckie     => "PL-MZ",
            PolandRegion::WojewodztwoOpolskie        => "PL-OP",
            PolandRegion::WojewodztwoPodkarpackie    => "PL-PK",
            PolandRegion::WojewodztwoPodlaskie       => "PL-PD",
            PolandRegion::WojewodztwoPomorskie       => "PL-PM",
            PolandRegion::WojewodztwoSlaskie         => "PL-SL",
            PolandRegion::WojewodztwoSwietokrzyskie  => "PL-SK",
            PolandRegion::WojewodztwoWarminskoMazurskie => "PL-WN",
            PolandRegion::WojewodztwoWielkopolskie   => "PL-WP",
            PolandRegion::WojewodztwoZachodniopomorskie => "PL-ZP",
        }
    }
}

impl TryFromAbbreviation for PolandRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "PL-DS" => Ok(PolandRegion::WojewodztwoDolnoslaskie),
            "PL-KP" => Ok(PolandRegion::WojewodztwoKujawskoPomorskie),
            "PL-LD" => Ok(PolandRegion::WojewodztwoLodzkie),
            "PL-LU" => Ok(PolandRegion::WojewodztwoLubelskie),
            "PL-LB" => Ok(PolandRegion::WojewodztwoLubuskie),
            "PL-MA" => Ok(PolandRegion::WojewodztwoMalopolskie),
            "PL-MZ" => Ok(PolandRegion::WojewodztwoMazowieckie),
            "PL-OP" => Ok(PolandRegion::WojewodztwoOpolskie),
            "PL-PK" => Ok(PolandRegion::WojewodztwoPodkarpackie),
            "PL-PD" => Ok(PolandRegion::WojewodztwoPodlaskie),
            "PL-PM" => Ok(PolandRegion::WojewodztwoPomorskie),
            "PL-SL" => Ok(PolandRegion::WojewodztwoSlaskie),
            "PL-SK" => Ok(PolandRegion::WojewodztwoSwietokrzyskie),
            "PL-WN" => Ok(PolandRegion::WojewodztwoWarminskoMazurskie),
            "PL-WP" => Ok(PolandRegion::WojewodztwoWielkopolskie),
            "PL-ZP" => Ok(PolandRegion::WojewodztwoZachodniopomorskie),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// RUSSIAN FEDERATION (Federal Districts, no official ISO macros)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for RussianFederationRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            RussianFederationRegion::CentralFederalDistrict    => "RU-CFD",
            RussianFederationRegion::CrimeanFederalDistrict    => "RU-CRF",
            RussianFederationRegion::FarEasternFederalDistrict => "RU-FEFD",
            RussianFederationRegion::NorthCaucasusFederalDistrict => "RU-NCFD",
            RussianFederationRegion::NorthwesternFederalDistrict => "RU-NWFD",
            RussianFederationRegion::SiberianFederalDistrict   => "RU-SIB",
            RussianFederationRegion::SouthFederalDistrict      => "RU-SFD",
            RussianFederationRegion::UralFederalDistrict       => "RU-UFD",
            RussianFederationRegion::VolgaFederalDistrict      => "RU-VFD",
        }
    }
}

impl TryFromAbbreviation for RussianFederationRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "RU-CFD"  => Ok(RussianFederationRegion::CentralFederalDistrict),
            "RU-CRF"  => Ok(RussianFederationRegion::CrimeanFederalDistrict),
            "RU-FEFD" => Ok(RussianFederationRegion::FarEasternFederalDistrict),
            "RU-NCFD" => Ok(RussianFederationRegion::NorthCaucasusFederalDistrict),
            "RU-NWFD" => Ok(RussianFederationRegion::NorthwesternFederalDistrict),
            "RU-SIB"  => Ok(RussianFederationRegion::SiberianFederalDistrict),
            "RU-SFD"  => Ok(RussianFederationRegion::SouthFederalDistrict),
            "RU-UFD"  => Ok(RussianFederationRegion::UralFederalDistrict),
            "RU-VFD"  => Ok(RussianFederationRegion::VolgaFederalDistrict),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// SPAIN (Autonomous Communities, official ISO 3166-2:ES)
///////////////////////////////////////////////////////////////////////////////

impl Abbreviation for SpainRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            SpainRegion::Andalucia        => "ES-AN",
            SpainRegion::Aragon           => "ES-AR",
            SpainRegion::Asturias         => "ES-AS",
            SpainRegion::Cantabria        => "ES-CB",
            SpainRegion::CastillaLaMancha => "ES-CM",
            SpainRegion::CastillaYLeon    => "ES-CL",
            SpainRegion::Cataluna         => "ES-CT",
            SpainRegion::Ceuta            => "ES-CE",
            SpainRegion::Extremadura      => "ES-EX",
            SpainRegion::Galicia          => "ES-GA",
            SpainRegion::IslasBaleares    => "ES-IB",
            SpainRegion::LaRioja          => "ES-RI",
            SpainRegion::Madrid           => "ES-MD",
            SpainRegion::Melilla          => "ES-ML",
            SpainRegion::Murcia           => "ES-MC",
            SpainRegion::Navarra          => "ES-NC",
            SpainRegion::PaisVasco        => "ES-PV",
            SpainRegion::Valencia         => "ES-VC",
        }
    }
}

impl TryFromAbbreviation for SpainRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "ES-AN" => Ok(SpainRegion::Andalucia),
            "ES-AR" => Ok(SpainRegion::Aragon),
            "ES-AS" => Ok(SpainRegion::Asturias),
            "ES-CB" => Ok(SpainRegion::Cantabria),
            "ES-CM" => Ok(SpainRegion::CastillaLaMancha),
            "ES-CL" => Ok(SpainRegion::CastillaYLeon),
            "ES-CT" => Ok(SpainRegion::Cataluna),
            "ES-CE" => Ok(SpainRegion::Ceuta),
            "ES-EX" => Ok(SpainRegion::Extremadura),
            "ES-GA" => Ok(SpainRegion::Galicia),
            "ES-IB" => Ok(SpainRegion::IslasBaleares),
            "ES-RI" => Ok(SpainRegion::LaRioja),
            "ES-MD" => Ok(SpainRegion::Madrid),
            "ES-ML" => Ok(SpainRegion::Melilla),
            "ES-MC" => Ok(SpainRegion::Murcia),
            "ES-NC" => Ok(SpainRegion::Navarra),
            "ES-PV" => Ok(SpainRegion::PaisVasco),
            "ES-VC" => Ok(SpainRegion::Valencia),
            _       => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// UNITED KINGDOM
///////////////////////////////////////////////////////////////////////////////

/// Top-level covers only England(… subregion?), Scotland, Wales.
/// Official ISO codes: GB-ENG, GB-SCT, GB-WLS. We'll assume
/// subregion `EnglandRegion` is also coded separately.
impl Abbreviation for UnitedKingdomRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            UnitedKingdomRegion::England(e) => e.abbreviation(),
            UnitedKingdomRegion::Scotland   => "GB-SCT",
            UnitedKingdomRegion::Wales      => "GB-WLS",
        }
    }
}

impl TryFromAbbreviation for UnitedKingdomRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        // If we can parse an England region:
        if let Ok(e) = EnglandRegion::try_from_abbreviation(abbr) {
            return Ok(UnitedKingdomRegion::England(e));
        }
        match abbr {
            "GB-SCT" => Ok(UnitedKingdomRegion::Scotland),
            "GB-WLS" => Ok(UnitedKingdomRegion::Wales),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}
