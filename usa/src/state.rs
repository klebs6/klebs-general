crate::ix!();

/// All US States (excluding D.C. since it's not a state).
/// 
/// We use strum attributes to handle various forms (full name, abbreviation, and spaceless name).
/// `ascii_case_insensitive` ensures case-insensitive matching.
/// `serialize_all = "title_case"` ensures variant names are considered in Title Case by default.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, StrumDisplay, StrumEnumString, StrumEnumIter, StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum UnitedState {
    #[strum(serialize = "Alabama",        serialize = "AL"                               )] Alabama,
    #[strum(serialize = "Alaska",         serialize = "AK"                               )] Alaska,
    #[strum(serialize = "Arizona",        serialize = "AZ"                               )] Arizona,
    #[strum(serialize = "Arkansas",       serialize = "AR"                               )] Arkansas,
    #[strum(serialize = "California",     serialize = "CA"                               )] California,
    #[strum(serialize = "Colorado",       serialize = "CO"                               )] Colorado,
    #[strum(serialize = "Connecticut",    serialize = "CT"                               )] Connecticut,
    #[strum(serialize = "Delaware",       serialize = "DE"                               )] Delaware,
    #[strum(serialize = "Florida",        serialize = "FL"                               )] Florida,
    #[strum(serialize = "Georgia",        serialize = "GA"                               )] Georgia,
    #[strum(serialize = "Hawaii",         serialize = "HI"                               )] Hawaii,
    #[strum(serialize = "Idaho",          serialize = "ID"                               )] Idaho,
    #[strum(serialize = "Illinois",       serialize = "IL"                               )] Illinois,
    #[strum(serialize = "Indiana",        serialize = "IN"                               )] Indiana,
    #[strum(serialize = "Iowa",           serialize = "IA"                               )] Iowa,
    #[strum(serialize = "Kansas",         serialize = "KS"                               )] Kansas,
    #[strum(serialize = "Kentucky",       serialize = "KY"                               )] Kentucky,
    #[strum(serialize = "Louisiana",      serialize = "LA"                               )] Louisiana,
    #[strum(serialize = "Maine",          serialize = "ME"                               )] Maine,
    #[strum(serialize = "Maryland",       serialize = "MD"                               )] Maryland,
    #[strum(serialize = "Massachusetts",  serialize = "MA"                               )] Massachusetts,
    #[strum(serialize = "Michigan",       serialize = "MI"                               )] Michigan,
    #[strum(serialize = "Minnesota",      serialize = "MN"                               )] Minnesota,
    #[strum(serialize = "Mississippi",    serialize = "MS"                               )] Mississippi,
    #[strum(serialize = "Missouri",       serialize = "MO"                               )] Missouri,
    #[strum(serialize = "Montana",        serialize = "MT"                               )] Montana,
    #[strum(serialize = "Nebraska",       serialize = "NE"                               )] Nebraska,
    #[strum(serialize = "Nevada",         serialize = "NV"                               )] Nevada,
    #[strum(serialize = "NewHampshire",   serialize = "New Hampshire",  serialize = "NH" )] NewHampshire,
    #[strum(serialize = "NewJersey",      serialize = "New Jersey",     serialize = "NJ" )] NewJersey,
    #[strum(serialize = "NewMexico",      serialize = "New Mexico",     serialize = "NM" )] NewMexico,
    #[strum(serialize = "NewYork",        serialize = "New York",       serialize = "NY" )] NewYork,
    #[strum(serialize = "NorthCarolina",  serialize = "North Carolina", serialize = "NC" )] NorthCarolina,
    #[strum(serialize = "NorthDakota",    serialize = "North Dakota",   serialize = "ND" )] NorthDakota,
    #[strum(serialize = "Ohio",           serialize = "OH"                               )] Ohio,
    #[strum(serialize = "Oklahoma",       serialize = "OK"                               )] Oklahoma,
    #[strum(serialize = "Oregon",         serialize = "OR"                               )] Oregon,
    #[strum(serialize = "Pennsylvania",   serialize = "PA"                               )] Pennsylvania,
    #[strum(serialize = "RhodeIsland",    serialize = "Rhode Island",   serialize = "RI" )] RhodeIsland,
    #[strum(serialize = "SouthCarolina",  serialize = "South Carolina", serialize = "SC" )] SouthCarolina,
    #[strum(serialize = "SouthDakota",    serialize = "South Dakota",   serialize = "SD" )] SouthDakota,
    #[strum(serialize = "Tennessee",      serialize = "TN"                               )] Tennessee,
    #[strum(serialize = "Texas",          serialize = "TX"                               )] Texas,
    #[strum(serialize = "Utah",           serialize = "UT"                               )] Utah,
    #[strum(serialize = "Vermont",        serialize = "VT"                               )] Vermont,
    #[strum(serialize = "Virginia",       serialize = "VA"                               )] Virginia,
    #[strum(serialize = "Washington",     serialize = "WA"                               )] Washington,
    #[strum(serialize = "WestVirginia",   serialize = "West Virginia",  serialize = "WV" )] WestVirginia,
    #[strum(serialize = "Wisconsin",      serialize = "WI"                               )] Wisconsin,
    #[strum(serialize = "Wyoming",        serialize = "WY"                               )] Wyoming,
}

impl UnitedState {

    pub fn all_states() -> Vec<UnitedState> {
        UnitedState::iter().collect()
    }
}

#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for UnitedState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for UnitedState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for UnitedState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        s.parse::<UnitedState>().map_err(|_| serde::de::Error::unknown_variant(&s, UnitedState::VARIANTS))
    }
}
