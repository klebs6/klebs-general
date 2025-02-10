// ---------------- [ File: src/create_address_stream_channel.rs ]
// ---------------- [ File: src/create_address_stream_channel.rs ]
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

#[cfg(test)]
mod create_address_stream_channel_tests {
    use super::*;
    use std::sync::mpsc::{TrySendError, TryRecvError};
    use std::thread;

    /// A helper to quickly produce a dummy `WorldAddress` or `OsmPbfParseError` for sending.
    fn dummy_world_address() -> WorldAddress {
        WorldAddressBuilder::default()
            .region(WorldRegion::default())
            .city(CityName::new("TestCity").unwrap())
            .street(StreetName::new("TestStreet").unwrap())
            .postal_code(PostalCode::new(Country::USA, "99999").unwrap())
            .build()
            .expect("Should build a dummy address")
    }

    /// Basic test: we can create the channel, send one item, receive it, and verify content.
    #[traced_test]
    fn test_create_address_stream_channel_basic_send_receive() {
        let (tx, rx) = create_address_stream_channel();

        // Send a dummy address
        let addr = dummy_world_address();
        tx.send(Ok(addr.clone())).expect("send should succeed");

        // Now receive
        let received = rx.recv().expect("recv should succeed");
        assert!(received.is_ok(), "Should yield an Ok(WorldAddress)");
        let unwrapped = received.unwrap();
        assert_eq!(unwrapped.city().name(), "testcity");
        assert_eq!(unwrapped.street().name(), "teststreet");
        assert_eq!(unwrapped.postal_code().code(), "99999");
    }

    /// Confirm we can send an `Err(OsmPbfParseError)` as well.
    #[traced_test]
    fn test_create_address_stream_channel_send_error() {
        let (tx, rx) = create_address_stream_channel();

        let error_val = OsmPbfParseError::InvalidInputFile {
            reason: "dummy reason".to_string(),
        };
        tx.send(Err(error_val)).expect("send error variant");
        let received = rx.recv().expect("recv error variant");
        assert!(received.is_err(), "Should yield an Err(...)");
        match received.err().unwrap() {
            OsmPbfParseError::InvalidInputFile { reason } => {
                assert_eq!(reason, "dummy reason");
            }
            other => panic!("Unexpected error: {:?}", other),
        }
    }

    /// Tests that the channel capacity is indeed 1000 by using `try_send(...)`.
    /// We can fill the channel up to 1000 sends, the 1001st should fail with `Full`.
    #[traced_test]
    fn test_create_address_stream_channel_capacity() {
        let (tx, _rx) = create_address_stream_channel();

        // We'll push 1000 items using try_send => all must succeed.
        for i in 0..1000 {
            let result = tx.try_send(Ok(dummy_world_address()));
            assert!(
                result.is_ok(),
                "Sending item {} should succeed within capacity 1000",
                i
            );
        }

        // Now the channel is presumably full => next try_send => Full error
        let attempt = tx.try_send(Ok(dummy_world_address()));
        match attempt {
            Err(TrySendError::Full(_)) => {
                // correct => channel is full
            }
            _ => panic!("Expected channel to be full at 1001st item"),
        }
    }

    /// Demonstrates that if we do receive from the channel, we free space and can send again.
    #[traced_test]
    fn test_create_address_stream_channel_send_receive_frees_space() {
        let (tx, rx) = create_address_stream_channel();

        // Fill the channel fully (1000 items)
        for _ in 0..1000 {
            tx.try_send(Ok(dummy_world_address())).expect("Should succeed up to 1000");
        }

        // Next item => Full
        let attempt = tx.try_send(Ok(dummy_world_address()));
        assert!(matches!(attempt, Err(TrySendError::Full(_))));

        // Now receive one => free 1 slot
        let received_one = rx.try_recv().expect("Should get at least one message");
        assert!(received_one.is_ok());
        
        // Next send => should succeed
        tx.try_send(Ok(dummy_world_address())).expect("Sending after 1 recv => should succeed");
    }
}
