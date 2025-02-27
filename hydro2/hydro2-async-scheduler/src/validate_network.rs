// ---------------- [ File: hydro2-async-scheduler/src/validate_network.rs ]
crate::ix!();

/// Validates the network by locking it and invoking `validate()`.
/// Returns an error if validation fails.
pub async fn validate_network<T>(
    network: &Arc<AsyncMutex<Network<T>>>,
) -> Result<(), NetworkError>
where
    T: Debug + Send + Sync,
{
    let net_guard = network.lock().await;
    eprintln!("execute_network: Validate network with {} nodes", net_guard.nodes().len());
    net_guard.validate()?;
    Ok(())
}

#[cfg(test)]
mod validate_network_tests {
    use super::*;

    #[test]
    fn test_validate_network_ok() {
        let rt = TokioRuntime::new().unwrap();
        rt.block_on(async {
            // Mock a network with valid structure
            let network = Arc::new(AsyncMutex::new(mock_valid_network()));
            let res = validate_network(&network).await;
            assert!(res.is_ok());
        });
    }
    
    // Helper stubs
    fn mock_valid_network() -> Network<u32> {
        // Construct a minimal valid network
        let mut net = Network::default();
        // ... set up valid nodes/edges ...
        net
    }
}
