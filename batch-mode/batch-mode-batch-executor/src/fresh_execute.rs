// ---------------- [ File: src/fresh_execute.rs ]
crate::ix!();

#[async_trait]
pub trait FreshExecute<Client,E> {
    type Success;
    async fn fresh_execute(&mut self, client: &Client) 
        -> Result<Self::Success, E>;
}
