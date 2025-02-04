crate::ix!();

/// Creates a bounded sync channel for streaming address results.
/// Returns `(SyncSender, Receiver)`.
pub fn create_address_stream_channel(
) -> (
    std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
    std::sync::mpsc::Receiver<Result<WorldAddress, OsmPbfParseError>>
) {
    // Capacity of 1000 is arbitrary; can be tweaked depending on performance needs.
    std::sync::mpsc::sync_channel(1000)
}
