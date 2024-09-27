crate::ix!();

/// Trait for creating instances asynchronously with input `X`
#[async_trait]
pub trait AsyncCreateWith<X> {
    type Error;

    async fn new(input: &X) -> Result<Self, Self::Error>
    where
        Self: Sized; // Ensure Self can be returned
}

/// Trait that combines async creation with integrity validation
#[async_trait]
pub trait AsyncCreateWithAndValidate<X>:
    Sized 
    + AsyncCreateWith<X> 
    + ValidateIntegrity<Error=<Self as AsyncCreateWith<X>>::Error> // Ensure same error type
    where for<'async_trait> X: Send + Sync + 'async_trait
{
    // Automatically forward the error type from AsyncCreateWith
    async fn new_and_validate(input: &X) -> Result<Self, <Self as AsyncCreateWith<X>>::Error> {
        let instance = Self::new(&input).await?;
        instance.validate_integrity()?; // Validation
        Ok(instance)
    }
}

// Blanket implementation of `AsyncCreateWithAndValidate` for types that implement
// both `AsyncCreateWith` and `ValidateIntegrity`
impl<X, T> AsyncCreateWithAndValidate<X> for T
where
    for<'async_trait> X: Send + Sync + 'async_trait,
    T: AsyncCreateWith<X> + ValidateIntegrity<Error=<T as AsyncCreateWith<X>>::Error>, // Ensure error type consistency
{
    // The methods from `AsyncCreateWithAndValidate` can be used here
}

#[async_trait]
pub trait AsyncIsValid {

    async fn is_valid(path: &Path) -> bool;
}

#[async_trait]
pub trait AsyncFindItemsFromPath {

    type Item;
    type Error;

    /// Asynchronously finds all the crates in the workspace
    async fn find_items(path: &Path) -> Result<Vec<Self::Item>, Self::Error>;
}
