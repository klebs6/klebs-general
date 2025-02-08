use std::path::Path;
use async_trait::async_trait;

// ---------------- [ File: src/async_traits.rs ]

pub trait MaybeThrow {

    type Error;

    fn maybe_throw(&self) -> Result<(),Self::Error>;
}

/// Trait for creating instances asynchronously with input `X`
#[async_trait]
pub trait AsyncTryFrom<X> {
    type Error;

    async fn new(input: &X) -> Result<Self, Self::Error>
    where
        Self: Sized; // Ensure Self can be returned
}

/// Trait that combines async creation with integrity validation
#[async_trait]
pub trait AsyncCreateWithAndValidate<X>:
    Sized 
    + AsyncTryFrom<X> 
    + ValidateIntegrity<Error=<Self as AsyncTryFrom<X>>::Error> // Ensure same error type
    where for<'async_trait> X: Send + Sync + 'async_trait
{
    // Automatically forward the error type from AsyncTryFrom
    async fn new_and_validate(input: &X) -> Result<Self, <Self as AsyncTryFrom<X>>::Error> {
        let instance = Self::new(&input).await?;
        instance.validate_integrity()?; // Validation
        Ok(instance)
    }
}

// Blanket implementation of `AsyncCreateWithAndValidate` for types that implement
// both `AsyncTryFrom` and `ValidateIntegrity`
impl<X, T> AsyncCreateWithAndValidate<X> for T
where
    for<'async_trait> X: Send + Sync + 'async_trait,
    T: AsyncTryFrom<X> + ValidateIntegrity<Error=<T as AsyncTryFrom<X>>::Error>, // Ensure error type consistency
{
    // The methods from `AsyncCreateWithAndValidate` can be used here
}

#[async_trait]
pub trait AsyncPathValidator {

    async fn is_valid(path: &Path) -> bool;
}

#[async_trait]
pub trait AsyncFindItems {

    type Item;
    type Error;

    /// Asynchronously finds all the crates in the workspace
    async fn find_items(path: &Path) -> Result<Vec<Self::Item>, Self::Error>;
}

/// Trait for validating integrity of a component (e.g., Workspace or Crate)
pub trait ValidateIntegrity {

    type Error;

    fn validate_integrity(&self) -> Result<(), Self::Error>;
}
