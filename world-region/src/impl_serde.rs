crate::ix!();

// Serialization/Deserialization
// We'll store as a map: {"continent": "<ContinentName>", "country": "<CountryName>" [,"region": "<SubregionName>"]}
#[cfg(not(feature = "serde_abbreviation"))]
impl Serialize for WorldRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let continent_str = match self {
            WorldRegion::Africa(_)                     => "Africa",
            WorldRegion::Asia(_)                       => "Asia",
            WorldRegion::Europe(_)                     => "Europe",
            WorldRegion::NorthAmerica(_)               => "North America",
            WorldRegion::SouthAmerica(_)               => "South America",
            WorldRegion::CentralAmerica(_)             => "Central America",
            WorldRegion::AustraliaOceaniaAntarctica(_) => "Australia/Oceania/Antarctica",
        };

        // Start by creating a Serializer map
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("continent", continent_str)?;

        // Now serialize the inner region separately and insert its keys
        let inner_json = match self {
            WorldRegion::Africa(r)                     => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::Asia(r)                       => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::Europe(r)                     => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::NorthAmerica(r)               => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::SouthAmerica(r)               => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::CentralAmerica(r)             => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
            WorldRegion::AustraliaOceaniaAntarctica(r) => serde_json::to_value(r).map_err(serde::ser::Error::custom)?,
        };

        // If the inner serialization produced an object, insert its keys into our map
        if let Some(obj) = inner_json.as_object() {
            for (k, v) in obj {
                map.serialize_entry(k, v).map_err(serde::ser::Error::custom)?;
            }
        }

        map.end()
    }
}

#[cfg(feature = "serde_abbreviation")]
impl Serialize for WorldRegion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        // If abbreviation feature is used, just serialize abbreviation:
        serializer.serialize_str(self.abbreviation())
    }
}

impl<'de> Deserialize<'de> for WorldRegion {
    fn deserialize<D>(deserializer: D) -> Result<WorldRegion, D::Error>
    where D: Deserializer<'de> {
        use serde_json::Value;
        let v = Value::deserialize(deserializer)?;

        let continent = v.get("continent").and_then(Value::as_str)
            .ok_or_else(|| DeError::missing_field("continent"))?;

        // We'll remove "continent" from the object and then try deserializing into the appropriate region enum.
        let mut map = v.as_object().cloned().ok_or_else(|| DeError::custom("Expected a map"))?;
        map.remove("continent");

        let value_without_continent = Value::Object(map);

        match continent {
            "Africa"                       => Ok(WorldRegion::Africa(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "Asia"                         => Ok(WorldRegion::Asia(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "Europe"                       => Ok(WorldRegion::Europe(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "North America"                => Ok(WorldRegion::NorthAmerica(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "South America"                => Ok(WorldRegion::SouthAmerica(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "Central America"              => Ok(WorldRegion::CentralAmerica(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            "Australia/Oceania/Antarctica" => Ok(WorldRegion::AustraliaOceaniaAntarctica(serde_json::from_value(value_without_continent).map_err(DeError::custom)?)),
            other                          => Err(DeError::unknown_variant(other, &[
                "Africa","Asia","Europe","North America","South America","Central America","Australia/Oceania/Antarctica"
            ]))
        }
    }
}
