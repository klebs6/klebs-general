crate::ix!();

#[async_trait]
pub trait FreshExecute<Client> {
    type Success;
    type Error;
    async fn fresh_execute(&mut self, client: &Client) 
        -> Result<Self::Success, Self::Error>;
}
