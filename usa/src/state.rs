crate::ix!();

/// All US States (excluding D.C. since it's not a state).
/// 
/// We use strum attributes to handle various forms (full name, abbreviation, and spaceless name).
/// `ascii_case_insensitive` ensures case-insensitive matching.
/// `serialize_all = "title_case"` ensures variant names are considered in Title Case by default.
#[derive(OsmPbfDownloader,Debug, PartialEq, Eq, Hash, Clone, Copy, StrumDisplay, StrumEnumString, StrumEnumIter, StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum UnitedState {

    #[geofabrik(link="north-america/us/alabama-latest.osm.pbf")]
    #[strum(serialize = "Alabama",        serialize = "AL"    )] 
    Alabama,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Alaska",         serialize = "AK"    )] 
    Alaska,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Arizona",        serialize = "AZ"    )] 
    Arizona,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Arkansas",       serialize = "AR"    )] 
    Arkansas,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "California",     serialize = "CA"    )] 
    California,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Colorado",       serialize = "CO"    )] 
    Colorado,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Connecticut",    serialize = "CT"    )] 
    Connecticut,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Delaware",       serialize = "DE"    )] 
    Delaware,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Florida",        serialize = "FL"    )] 
    Florida,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Georgia",        serialize = "GA"    )] 
    Georgia,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Hawaii",         serialize = "HI"    )] 
    Hawaii,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Idaho",          serialize = "ID"    )] 
    Idaho,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Illinois",       serialize = "IL"    )] 
    Illinois,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Indiana",        serialize = "IN"    )] 
    Indiana,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Iowa",           serialize = "IA"    )] 
    Iowa,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Kansas",         serialize = "KS"    )] 
    Kansas,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Kentucky",       serialize = "KY"    )] 
    Kentucky,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Louisiana",      serialize = "LA"    )] 
    Louisiana,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Maine",          serialize = "ME"    )] 
    Maine,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Maryland",       serialize = "MD"    )] 
    Maryland,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Massachusetts",  serialize = "MA"    )] 
    Massachusetts,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Michigan",       serialize = "MI"    )] 
    Michigan,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Minnesota",      serialize = "MN"    )] 
    Minnesota,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Mississippi",    serialize = "MS"    )] 
    Mississippi,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Missouri",       serialize = "MO"    )] 
    Missouri,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Montana",        serialize = "MT"    )] 
    Montana,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Nebraska",       serialize = "NE"    )] 
    Nebraska,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Nevada",         serialize = "NV"    )] 
    Nevada,

    #[geofabrik(link="")]
    #[strum(serialize = "NewHampshire",   serialize = "New Hampshire",  serialize = "NH" )] 
    NewHampshire,

    #[geofabrik(link="")]
    #[strum(serialize = "NewJersey",      serialize = "New Jersey",     serialize = "NJ" )] 
    NewJersey,

    #[geofabrik(link="")]
    #[strum(serialize = "NewMexico",      serialize = "New Mexico",     serialize = "NM" )] 
    NewMexico,

    #[geofabrik(link="")]
    #[strum(serialize = "NewYork",        serialize = "New York",       serialize = "NY" )] 
    NewYork,

    #[geofabrik(link="")]
    #[strum(serialize = "NorthCarolina",  serialize = "North Carolina", serialize = "NC" )] 
    NorthCarolina,

    #[geofabrik(link="")]
    #[strum(serialize = "NorthDakota",    serialize = "North Dakota",   serialize = "ND" )] 
    NorthDakota,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Ohio",           serialize = "OH"    )] 
    Ohio,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Oklahoma",       serialize = "OK"    )] 
    Oklahoma,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Oregon",         serialize = "OR"    )] 
    Oregon,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Pennsylvania",   serialize = "PA"    )] 
    Pennsylvania,

    #[geofabrik(link="")]
    #[strum(serialize = "RhodeIsland",    serialize = "Rhode Island",   serialize = "RI" )] 
    RhodeIsland,

    #[geofabrik(link="")]
    #[strum(serialize = "SouthCarolina",  serialize = "South Carolina", serialize = "SC" )] 
    SouthCarolina,

    #[geofabrik(link="")]
    #[strum(serialize = "SouthDakota",    serialize = "South Dakota",   serialize = "SD" )] 
    SouthDakota,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Tennessee",      serialize = "TN"    )] 
    Tennessee,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Texas",          serialize = "TX"    )] 
    Texas,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Utah",           serialize = "UT"    )] 
    Utah,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Vermont",        serialize = "VT"    )] 
    Vermont,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Virginia",       serialize = "VA"    )] 
    Virginia,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Washington",     serialize = "WA"    )] 
    Washington,

    #[geofabrik(link="")]
    #[strum(serialize = "WestVirginia",   serialize = "West Virginia",  serialize = "WV" )] 
    WestVirginia,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Wisconsin",      serialize = "WI"    )] 
    Wisconsin,

    #[geofabrik(link=""                                       )]
    #[strum(serialize = "Wyoming",        serialize = "WY"    )] 
    Wyoming,
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
