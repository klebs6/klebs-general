// ---------------- [ File: src/drain_channel.rs ]
crate::ix!();

/// Drains a `Receiver<usize>` until `None`, returning items read.
pub async fn drain_channel<T>(mut rx: Receiver<T>) -> Vec<T> {
    let mut out = Vec::new();
    while let Some(v) = rx.recv().await {
        out.push(v);
    }
    out
}

#[cfg(test)]
mod drain_channel_tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::{timeout, Duration};

    #[traced_test]
    async fn test_drain_channel_non_empty_channel() {
        let (tx, rx) = mpsc::channel(10);
        
        // Send some items to the channel
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
        drop(tx); // Close the sender

        // Drain the channel
        let result = drain_channel(rx).await;

        // Assert the result
        assert_eq!(result, vec![0, 1, 2, 3, 4]);
    }

    #[traced_test]
    async fn test_drain_channel_empty_channel() {
        let (_tx, rx) = mpsc::channel::<usize>(10);

        drop(_tx);

        // Drain the channel
        let result = drain_channel(rx).await;

        // Assert the result
        assert!(result.is_empty());
    }

    #[traced_test]
    async fn test_drain_channel_no_items_channel_closed() {
        let (_tx, rx) = mpsc::channel::<usize>(10);
        drop(_tx); // Explicitly drop the sender to close the channel

        // Drain the channel
        let result = drain_channel(rx).await;

        // Assert the result
        assert!(result.is_empty());
    }

    #[traced_test]
    async fn test_drain_channel_partial_read() {
        let (tx, rx) = mpsc::channel(10);

        // Send some items to the channel
        for i in 0..3 {
            tx.send(i).await.unwrap();
        }

        // Drop the sender before sending more
        drop(tx);

        // Drain the channel
        let result = drain_channel(rx).await;

        // Assert the result
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[traced_test]
    async fn test_drain_channel_with_timeout() {
        let (tx, rx) = mpsc::channel(10);

        // Spawn a task to send items with delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            tx.send(42).await.unwrap();
            drop(tx); // Close the sender
        });

        // Drain the channel with timeout
        let result = timeout(Duration::from_secs(1), drain_channel(rx)).await;

        // Assert the result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![42]);
    }

    #[traced_test]
    async fn test_drain_channel_high_capacity() {
        let (tx, rx) = mpsc::channel(100);

        // Send a large number of items
        for i in 0..100 {
            tx.send(i).await.unwrap();
        }
        drop(tx); // Close the sender

        // Drain the channel
        let result = drain_channel(rx).await;

        // Assert the result
        assert_eq!(result, (0..100).collect::<Vec<_>>());
    }
}
