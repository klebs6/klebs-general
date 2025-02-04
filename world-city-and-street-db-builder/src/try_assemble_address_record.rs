// ---------------- [ File: src/try_assemble_address_record.rs ]
crate::ix!();

/// Assembles the final `AddressRecord` using the provided city, street, and postcode objects.
/// Returns an error if the `AddressRecordBuilder` fails to build the record.
pub fn try_assemble_address_record(
    city: Option<CityName>,
    street: Option<StreetName>,
    postcode: Option<PostalCode>,
    element_id: i64,
) -> Result<AddressRecord, IncompatibleOsmPbfElement> {
    trace!("try_assemble_address_record: Building AddressRecord for element_id={}", element_id);
    AddressRecordBuilder::default()
        .city(city)
        .street(street)
        .postcode(postcode)
        .build()
        .map_err(|builder_error| {
            error!(
                "try_assemble_address_record: Builder error for element_id={}: {:?}",
                element_id, builder_error
            );
            IncompatibleOsmPbfElement::IncompatibleOsmPbfNode(
                IncompatibleOsmPbfNode::AddressRecordBuilderError {
                    id: element_id,
                    source: builder_error,
                }
            )
        })
}
