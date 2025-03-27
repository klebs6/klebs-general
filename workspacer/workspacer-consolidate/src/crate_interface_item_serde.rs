// ---------------- [ File: workspacer-consolidate/src/crate_interface_item_serde.rs ]
crate::ix!();

impl<T> ::serde::Serialize for CrateInterfaceItem<T>
where
    T: RehydrateFromSignature,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        trace!("Serializing CrateInterfaceItem<T> manually.");

        // We have five fields to serialize:
        //   item_signature (String)      => from `self.item.generate_signature()`
        //   docs (Option<String>)
        //   attributes (Option<String>)
        //   body_source (Option<String>)
        //   consolidation_options (Option<ConsolidationOptions>)
        let mut state = serializer.serialize_struct("CrateInterfaceItem", 5)?;

        // 1) item_signature => always present
        let signature_str = self.item().generate_signature();
        state.serialize_field("item_signature", &signature_str)?;

        // 2) docs
        state.serialize_field("docs", &self.docs())?;

        // 3) attributes
        state.serialize_field("attributes", &self.attributes())?;

        // 4) body_source
        state.serialize_field("body_source", &self.body_source())?;

        // 5) consolidation_options
        state.serialize_field("consolidation_options", &self.consolidation_options())?;

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
        debug!("Deserializing CrateInterfaceItem<T> via a custom Visitor.");
        
        // The fields we expect:
        const FIELDS: &[&str] = &[
            "item_signature",
            "docs",
            "attributes",
            "body_source",
            "consolidation_options",
        ];

        // We define an enum to match the field names:
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ItemSignature,
            Docs,
            Attributes,
            BodySource,
            ConsolidationOptions,
        }

        // This Visitor will build our final CrateInterfaceItem<T>.
        struct CrateInterfaceItemVisitor<U: RehydrateFromSignature> {
            marker: PhantomData<fn() -> CrateInterfaceItem<U>>,
        }

        impl<'de, U> Visitor<'de> for CrateInterfaceItemVisitor<U>
        where
            U: RehydrateFromSignature,
        {
            type Value = CrateInterfaceItem<U>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "struct CrateInterfaceItem")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                trace!("Visiting map entries to deserialize CrateInterfaceItem<T>.");

                // We'll store each field in a local Option<...> variable until we have them all.
                let mut item_signature: Option<String> = None;
                let mut docs: Option<Option<String>> = None;
                let mut attributes: Option<Option<String>> = None;
                let mut body_source: Option<Option<String>> = None;
                let mut consolidation_opts: Option<Option<ConsolidationOptions>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::ItemSignature => {
                            // required => parse as a plain String
                            let sig: String = map.next_value()?;
                            item_signature = Some(sig);
                        }
                        Field::Docs => {
                            // optional => parse as Option<String>
                            docs = Some(map.next_value::<Option<String>>()?);
                        }
                        Field::Attributes => {
                            attributes = Some(map.next_value::<Option<String>>()?);
                        }
                        Field::BodySource => {
                            body_source = Some(map.next_value::<Option<String>>()?);
                        }
                        Field::ConsolidationOptions => {
                            // parse as Option<ConsolidationOptions>, so null => None
                            consolidation_opts = Some(map.next_value::<Option<ConsolidationOptions>>()?);
                        }
                    }
                }

                // item_signature is required:
                let item_signature =
                    item_signature.ok_or_else(|| DeError::missing_field("item_signature"))?;

                // Rehydrate T from the signature string:
                let rehydrated_item = U::rehydrate_from_signature(&item_signature).ok_or_else(|| {
                    DeError::custom(format!("Failed to rehydrate `T` from signature: `{}`", item_signature))
                })?;

                trace!("Successfully rehydrated T from signature.");

                // Convert double-option to single
                let final_docs = docs.unwrap_or(None);
                let final_attrs = attributes.unwrap_or(None);
                let final_body = body_source.unwrap_or(None);
                let final_co = consolidation_opts.unwrap_or(None);

                // Build via the builder or directly:
                let item = CrateInterfaceItemBuilder::default()
                    .item(Arc::new(rehydrated_item))
                    .docs(final_docs)
                    .attributes(final_attrs)
                    .body_source(final_body)
                    .consolidation_options(final_co)
                    .build()
                    .map_err(DeError::custom)?;

                Ok(item)
            }
        }

        // Actually invoke the visitor:
        deserializer.deserialize_struct(
            "CrateInterfaceItem",
            FIELDS,
            CrateInterfaceItemVisitor {
                marker: PhantomData,
            },
        )
    }
}
