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
            EuropeRegion::France(_)            => "FR",
            EuropeRegion::Germany(_)           => "DE",
            EuropeRegion::Italy(_)             => "IT",
            EuropeRegion::Netherlands(_)       => "NL",
            EuropeRegion::Poland(_)            => "PL",
            EuropeRegion::RussianFederation(_) => "RU",
            EuropeRegion::Spain(_)             => "ES",
            EuropeRegion::UnitedKingdom(_)     => "GB",
        }
    }
}
