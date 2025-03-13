crate::ix!();

impl<T> ::serde::Serialize for CrateInterfaceItem<T>
where
    T: RehydrateFromSignature,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        trace!("Serializing CrateInterfaceItem<T> directly (no helper struct).");

        let mut state = serializer.serialize_struct("CrateInterfaceItem", 5)?;
        state.serialize_field("item_signature", &self.item.generate_signature())?;
        state.serialize_field("docs", &self.docs)?;
        state.serialize_field("attributes", &self.attributes)?;
        state.serialize_field("body_source", &self.body_source)?;
        state.serialize_field("consolidation_options", &self.consolidation_options)?;
        state.end()
    }
}

impl<'de, T> ::serde::Deserialize<'de> for CrateInterfaceItem<T>
where
    T: RehydrateFromSignature,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        // The fields we expect in our serialized representation:
        const FIELDS: &[&str] = &[
            "item_signature",
            "docs",
            "attributes",
            "body_source",
            "consolidation_options",
        ];

        // We'll define a tiny field identifier enum:
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ItemSignature,
            Docs,
            Attributes,
            BodySource,
            ConsolidationOptions,
        }

        // Our Visitor will reconstruct the final `CrateInterfaceItem<T>`.
        struct CrateInterfaceItemVisitor<T> {
            marker: PhantomData<fn() -> CrateInterfaceItem<T>>,
        }

        impl<'de, U> Visitor<'de> for CrateInterfaceItemVisitor<U>
        where
            U: RehydrateFromSignature,
        {
            type Value = CrateInterfaceItem<U>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "struct CrateInterfaceItem")
            }

            #[tracing::instrument(level = "trace", skip(self, map))]
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                trace!("Visiting map to deserialize CrateInterfaceItem<T>.");

                let mut item_signature: Option<String> = None;
                let mut docs: Option<String> = None;
                let mut attributes: Option<String> = None;
                let mut body_source: Option<String> = None;
                let mut consolidation_options: Option<ConsolidationOptions> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ItemSignature => {
                            item_signature = Some(map.next_value()?);
                        }
                        Field::Docs => {
                            docs = Some(map.next_value()?);
                        }
                        Field::Attributes => {
                            attributes = Some(map.next_value()?);
                        }
                        Field::BodySource => {
                            body_source = Some(map.next_value()?);
                        }
                        Field::ConsolidationOptions => {
                            consolidation_options = Some(map.next_value()?);
                        }
                    }
                }

                let item_signature = item_signature
                    .ok_or_else(|| DeserError::missing_field("item_signature"))?;

                let rehydrated_item = U::rehydrate_from_signature(&item_signature).ok_or_else(|| {
                    DeserError::custom(format!(
                        "Failed to rehydrate `T` from signature: `{item_signature}`"
                    ))
                })?;

                trace!("Rehydration succeeded; constructing final CrateInterfaceItem.");

                Ok(CrateInterfaceItem {
                    item: Arc::new(rehydrated_item),
                    docs,
                    attributes,
                    body_source,
                    consolidation_options,
                })
            }
        }

        debug!("Calling deserialize_struct for CrateInterfaceItem<T>.");
        deserializer.deserialize_struct(
            "CrateInterfaceItem",
            FIELDS,
            CrateInterfaceItemVisitor {
                marker: PhantomData,
            },
        )
    }
}
