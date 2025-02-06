crate::ix!();

/// All US States (excluding D.C. since it's not a state).
/// 
/// We use strum attributes to handle various forms (full name, abbreviation, and spaceless name).
/// `ascii_case_insensitive` ensures case-insensitive matching.
/// `serialize_all = "title_case"` ensures variant names are considered in Title Case by default.
#[derive(OsmPbfFileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumString,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive, serialize_all = "title_case")]
pub enum UnitedState {

    #[geofabrik(usa="alabama-latest.osm.pbf")] 
    #[strum(serialize = "Alabama", serialize = "AL"        )] 
    Alabama,

    #[geofabrik(usa="alaska-latest.osm.pbf")] 
    #[strum(serialize = "Alaska",         serialize = "AK" )] 
    Alaska,

    #[geofabrik(usa="arizona-latest.osm.pbf")] 
    #[strum(serialize = "Arizona",        serialize = "AZ" )] 
    Arizona,

    #[geofabrik(usa="arkansas-latest.osm.pbf")] 
    #[strum(serialize = "Arkansas",       serialize = "AR" )] 
    Arkansas,

    #[geofabrik(usa="california-latest.osm.pbf")] 
    #[strum(serialize = "California",     serialize = "CA" )] 
    California,

    #[geofabrik(usa="colorado-latest.osm.pbf")] 
    #[strum(serialize = "Colorado",       serialize = "CO" )] 
    Colorado,

    #[geofabrik(usa="connecticut-latest.osm.pbf")] 
    #[strum(serialize = "Connecticut",    serialize = "CT" )] 
    Connecticut,

    #[geofabrik(usa="delaware-latest.osm.pbf")] 
    #[strum(serialize = "Delaware",       serialize = "DE" )] 
    Delaware,

    #[geofabrik(usa="florida-latest.osm.pbf")] 
    #[strum(serialize = "Florida",        serialize = "FL" )] 
    Florida,

    #[geofabrik(usa="georgia-latest.osm.pbf")] 
    #[strum(serialize = "Georgia",        serialize = "GA" )] 
    Georgia,

    #[geofabrik(usa="hawaii-latest.osm.pbf")] 
    #[strum(serialize = "Hawaii",         serialize = "HI" )] 
    Hawaii,

    #[geofabrik(usa="idaho-latest.osm.pbf")] 
    #[strum(serialize = "Idaho",          serialize = "ID" )] 
    Idaho,

    #[geofabrik(usa="illinois-latest.osm.pbf")] 
    #[strum(serialize = "Illinois",       serialize = "IL" )] 
    Illinois,

    #[geofabrik(usa="indiana-latest.osm.pbf")] 
    #[strum(serialize = "Indiana",        serialize = "IN" )] 
    Indiana,

    #[geofabrik(usa="iowa-latest.osm.pbf")] 
    #[strum(serialize = "Iowa",           serialize = "IA" )] 
    Iowa,

    #[geofabrik(usa="kansas-latest.osm.pbf")] 
    #[strum(serialize = "Kansas",         serialize = "KS" )] 
    Kansas,

    #[geofabrik(usa="kentucky-latest.osm.pbf")] 
    #[strum(serialize = "Kentucky",       serialize = "KY" )] 
    Kentucky,

    #[geofabrik(usa="louisiana-latest.osm.pbf")] 
    #[strum(serialize = "Louisiana",      serialize = "LA" )] 
    Louisiana,

    #[geofabrik(usa="maine-latest.osm.pbf")] 
    #[strum(serialize = "Maine",          serialize = "ME" )] 
    Maine,

    #[geofabrik(usa="maryland-latest.osm.pbf")] 
    #[strum(serialize = "Maryland",       serialize = "MD" )] 
    Maryland,

    #[geofabrik(usa="massachusetts-latest.osm.pbf")] 
    #[strum(serialize = "Massachusetts",  serialize = "MA" )] 
    Massachusetts,

    #[geofabrik(usa="michigan-latest.osm.pbf")] 
    #[strum(serialize = "Michigan",       serialize = "MI" )] 
    Michigan,

    #[geofabrik(usa="minnesota-latest.osm.pbf")] 
    #[strum(serialize = "Minnesota",      serialize = "MN" )] 
    Minnesota,

    #[geofabrik(usa="mississippi-latest.osm.pbf")] 
    #[strum(serialize = "Mississippi",    serialize = "MS" )] 
    Mississippi,

    #[geofabrik(usa="missouri-latest.osm.pbf")] 
    #[strum(serialize = "Missouri",       serialize = "MO" )] 
    Missouri,

    #[geofabrik(usa="montana-latest.osm.pbf")] 
    #[strum(serialize = "Montana",        serialize = "MT" )] 
    Montana,

    #[geofabrik(usa="nebraska-latest.osm.pbf")] 
    #[strum(serialize = "Nebraska",       serialize = "NE" )] 
    Nebraska,

    #[geofabrik(usa="nevada-latest.osm.pbf")] 
    #[strum(serialize = "Nevada",         serialize = "NV" )] 
    Nevada,

    #[geofabrik(usa="new-hampshire-latest.osm.pbf")] 
    #[strum(serialize = "NewHampshire",   serialize = "New Hampshire",  serialize = "NH" )] 
    NewHampshire,

    #[geofabrik(usa="new-jersey-latest.osm.pbf")] 
    #[strum(serialize = "NewJersey",      serialize = "New Jersey",     serialize = "NJ" )] 
    NewJersey,

    #[geofabrik(usa="new-mexico-latest.osm.pbf")] 
    #[strum(serialize = "NewMexico",      serialize = "New Mexico",     serialize = "NM" )] 
    NewMexico,

    #[geofabrik(usa="new-york-latest.osm.pbf")] 
    #[strum(serialize = "NewYork",        serialize = "New York",       serialize = "NY" )] 
    NewYork,

    #[geofabrik(usa="north-carolina-latest.osm.pbf")] 
    #[strum(serialize = "NorthCarolina",  serialize = "North Carolina", serialize = "NC" )] 
    NorthCarolina,

    #[geofabrik(usa="north-dakota-latest.osm.pbf")] 
    #[strum(serialize = "NorthDakota",    serialize = "North Dakota",   serialize = "ND" )] 
    NorthDakota,

    #[geofabrik(usa="ohio-latest.osm.pbf")] 
    #[strum(serialize = "Ohio",           serialize = "OH" )] 
    Ohio,

    #[geofabrik(usa="oklahoma-latest.osm.pbf")] 
    #[strum(serialize = "Oklahoma",       serialize = "OK" )] 
    Oklahoma,

    #[geofabrik(usa="oregon-latest.osm.pbf")] 
    #[strum(serialize = "Oregon",         serialize = "OR" )] 
    Oregon,

    #[geofabrik(usa="pennsylvania-latest.osm.pbf")] 
    #[strum(serialize = "Pennsylvania",   serialize = "PA" )] 
    Pennsylvania,

    #[geofabrik(usa="rhode-island-latest.osm.pbf")] 
    #[strum(serialize = "RhodeIsland",    serialize = "Rhode Island",   serialize = "RI" )] 
    RhodeIsland,

    #[geofabrik(usa="south-carolina-latest.osm.pbf")] 
    #[strum(serialize = "SouthCarolina",  serialize = "South Carolina", serialize = "SC" )] 
    SouthCarolina,

    #[geofabrik(usa="south-dakota-latest.osm.pbf")] 
    #[strum(serialize = "SouthDakota",    serialize = "South Dakota",   serialize = "SD" )] 
    SouthDakota,

    #[geofabrik(usa="tennessee-latest.osm.pbf")] 
    #[strum(serialize = "Tennessee",      serialize = "TN" )] 
    Tennessee,

    #[geofabrik(usa="texas-latest.osm.pbf")] 
    #[strum(serialize = "Texas",          serialize = "TX" )] 
    Texas,

    #[geofabrik(usa="utah-latest.osm.pbf")] 
    #[strum(serialize = "Utah",           serialize = "UT" )] 
    Utah,

    #[geofabrik(usa="vermont-latest.osm.pbf")] 
    #[strum(serialize = "Vermont",        serialize = "VT" )] 
    Vermont,

    #[geofabrik(usa="virginia-latest.osm.pbf")] 
    #[strum(serialize = "Virginia",       serialize = "VA" )] 
    Virginia,

    #[geofabrik(usa="washington-latest.osm.pbf")] 
    #[strum(serialize = "Washington",     serialize = "WA" )] 
    Washington,

    #[geofabrik(usa="west-virginia-latest.osm.pbf")] 
    #[strum(serialize = "WestVirginia",   serialize = "West Virginia",  serialize = "WV" )] 
    WestVirginia,

    #[geofabrik(usa="wisconsin-latest.osm.pbf")] 
    #[strum(serialize = "Wisconsin",      serialize = "WI" )] 
    Wisconsin,

    #[geofabrik(usa="wyoming-latest.osm.pbf")] 
    #[strum(serialize = "Wyoming",        serialize = "WY" )] 
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
