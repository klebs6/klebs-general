// ---------------- [ File: src/save_load_traits.rs ]
crate::ix!();

#[async_trait]
pub trait SaveToFile {

    type Error;

    async fn save_to_file(
        &self,
        filename: impl AsRef<Path> + Send,

    ) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait LoadFromFile: Sized {

    type Error;

    async fn load_from_file(filename: impl AsRef<Path> + Send) 
        -> Result<Self, Self::Error>;

}
