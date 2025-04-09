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

        // We have six fields to serialize now:
        //   1) item_signature (String) => from `self.item.generate_signature()`
        //   2) docs (Option<String>)
        //   3) attributes (Option<String>)
        //   4) body_source (Option<String>)
        //   5) consolidation_options (Option<ConsolidationOptions>)
        //   6) raw_syntax_range (TextRange)
        //   7) effective_item_range (TextRange)
        let mut state = serializer.serialize_struct("CrateInterfaceItem", 7)?;

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

        // 6) raw_syntax_range
        state.serialize_field("raw_syntax_range", &self.raw_syntax_range())?;

        // 7) effective_item_range
        state.serialize_field("effective_item_range", &self.effective_range())?;

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
        // We expect 7 total fields
        const FIELDS: &[&str] = &[
            "item_signature",
            "docs",
            "attributes",
            "body_source",
            "consolidation_options",
            "raw_syntax_range",
            "effective_item_range",
        ];

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ItemSignature,
            Docs,
            Attributes,
            BodySource,
            ConsolidationOptions,
            RawSyntaxRange,
            EffectiveItemRange,
        }

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
                let mut item_signature:     Option<String>                       = None;
                let mut docs:               Option<Option<String>>               = None;
                let mut attributes:         Option<Option<String>>               = None;
                let mut body_source:        Option<Option<String>>               = None;
                let mut consolidation_opts: Option<Option<ConsolidationOptions>> = None;
                let mut raw_range:          Option<TextRange>                    = None;
                let mut eff_range:          Option<TextRange>                    = None;

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
                            consolidation_opts = Some(map.next_value()?);
                        }
                        Field::RawSyntaxRange => {
                            raw_range = Some(map.next_value()?);
                        }
                        Field::EffectiveItemRange => {
                            eff_range = Some(map.next_value()?);
                        }
                    }
                }

                let item_signature =
                    item_signature.ok_or_else(|| DeError::missing_field("item_signature"))?;
                let rehydrated_item = U::rehydrate_from_signature(&item_signature)
                    .ok_or_else(|| DeError::custom(format!("Failed to rehydrate `T` from: {}", item_signature)))?;

                let final_docs       = docs.unwrap_or(None);
                let final_attrs      = attributes.unwrap_or(None);
                let final_body       = body_source.unwrap_or(None);
                let final_co         = consolidation_opts.unwrap_or(None);

                let raw_syntax_range = raw_range.ok_or_else(|| DeError::missing_field("raw_syntax_range"))?;
                let effective_range  = eff_range.ok_or_else(|| DeError::missing_field("effective_item_range"))?;

                // Build:
                Ok(
                    CrateInterfaceItem::new_with_paths_and_ranges(
                        rehydrated_item,
                        final_docs,
                        final_attrs,
                        final_body,
                        final_co,
                        PathBuf::new(), // we store an empty path, or you can store some placeholder
                        PathBuf::new(),
                        raw_syntax_range,
                        effective_range,
                    )
                )
            }
        }

        deserializer.deserialize_struct(
            "CrateInterfaceItem",
            FIELDS,
            CrateInterfaceItemVisitor { marker: PhantomData },
        )
    }
}
