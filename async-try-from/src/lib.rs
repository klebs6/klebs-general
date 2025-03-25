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
        instance.validate_integrity().await?; // Validation
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
#[async_trait]
pub trait ValidateIntegrity {

    type Error;

    async fn validate_integrity(&self) -> Result<(), Self::Error>;
}

/// Trait for asynchronously creating `Self` specifically from an environment variable name.
#[async_trait]
pub trait AsyncTryFromEnv {
    type Error;

    /// Creates an instance of `Self` by reading an environment variable asynchronously (if needed).
    async fn new_from_env(var_name: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Trait for asynchronously creating `Self` from a filesystem path.
#[async_trait]
pub trait AsyncTryFromFile {
    type Error;

    /// Asynchronously create `Self` by reading the file at `path`.
    async fn new_from_file(path: &Path) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Trait that combines the environment creation with `ValidateIntegrity`.
#[async_trait]
pub trait AsyncCreateWithAndValidateEnv:
    Sized
    + AsyncTryFromEnv
    + ValidateIntegrity<Error = <Self as AsyncTryFromEnv>::Error>
{
    async fn new_from_env_and_validate(var_name: &str) -> Result<Self, <Self as AsyncTryFromEnv>::Error> {
        let instance = Self::new_from_env(var_name).await?;
        instance.validate_integrity().await?;
        Ok(instance)
    }
}

// Blanket implementation if a type implements both `AsyncTryFromEnv` and `ValidateIntegrity`
impl<T> AsyncCreateWithAndValidateEnv for T
where
    T: AsyncTryFromEnv + ValidateIntegrity<Error = <T as AsyncTryFromEnv>::Error>,
{}

/// Trait that combines file-based creation with `ValidateIntegrity`.
#[async_trait]
pub trait AsyncCreateWithAndValidateFile:
    Sized
    + AsyncTryFromFile
    + ValidateIntegrity<Error = <Self as AsyncTryFromFile>::Error>
{
    async fn new_from_file_and_validate(path: &Path) -> Result<Self, <Self as AsyncTryFromFile>::Error> {
        let instance = Self::new_from_file(path).await?;
        instance.validate_integrity().await?;
        Ok(instance)
    }
}

// Blanket implementation if a type implements both `AsyncTryFromFile` and `ValidateIntegrity`
impl<T> AsyncCreateWithAndValidateFile for T
where
    T: AsyncTryFromFile + ValidateIntegrity<Error = <T as AsyncTryFromFile>::Error>,
{}
