crate::ix!();

/// Trait for checking if a component is ready for Cargo publishing
#[async_trait]
pub trait ReadyForCargoPublish {
    type Error;
    async fn ready_for_cargo_publish(&self) -> Result<(),Self::Error>;
}
