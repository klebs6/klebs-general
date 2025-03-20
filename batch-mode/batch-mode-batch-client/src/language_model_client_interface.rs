// ---------------- [ File: src/language_model_client_interface.rs ]
crate::ix!();

/*
   To make a trait object-safe, we cannot have any methods that use generics
   (`impl Trait` or a type parameter in the method signature). Previously,
   `UploadBatchFile` included `upload_batch_file<P>`, causing object safety issues.

   The fix is to split the functionality into:

   1) An object-safe "core" trait (no generic methods).
   2) An optional "extension" trait that adds the generic convenience method.

   Then our aggregator trait references only the *core* trait, ensuring no
   generic methods are included, which makes it object-safe.
*/

/* ================================
   Sub-traits: object-safe methods
   ================================
*/

#[async_trait]
pub trait RetrieveBatchById: Send + Sync {
    type Error;
    async fn retrieve_batch(&self, batch_id: &str) -> Result<Batch, Self::Error>;
}

#[async_trait]
pub trait GetBatchFileContent: Send + Sync {
    type Error;
    async fn file_content(&self, file_id: &str) -> Result<Bytes, Self::Error>;
}

/*
   "Core" trait for uploading a file, object-safe:
   no generic parameters or `impl Trait` in the signature.
*/
#[async_trait]
pub trait UploadBatchFileCore: Send + Sync {
    type Error;

    async fn upload_batch_file_path(
        &self,
        file_path: &Path
    ) -> Result<OpenAIFile, Self::Error>;
}

/*
   OPTIONAL "extension" trait that provides a convenience
   generic method. This trait is NOT object-safe, but doesn't
   need to be used as a trait object. It is purely for convenience.
*/
#[async_trait]
pub trait UploadBatchFileExt: UploadBatchFileCore {
    async fn upload_batch_file<P>(
        &self,
        file_path: P
    ) -> Result<OpenAIFile, Self::Error>
    where
        P: AsRef<Path> + Send + Sync,
    {
        // Default implementation simply calls the object-safe method
        self.upload_batch_file_path(file_path.as_ref()).await
    }
}

#[async_trait]
pub trait CreateBatch: Send + Sync {
    type Error;
    async fn create_batch(
        &self,
        input_file_id: &str,
    ) -> Result<Batch, Self::Error>;
}

#[async_trait]
pub trait WaitForBatchCompletion: Send + Sync {
    type Error;
    async fn wait_for_batch_completion(
        &self,
        batch_id: &str,
    ) -> Result<Batch, Self::Error>;
}

/*
   =========================================================
   Aggregator trait referencing ONLY the object-safe methods
   =========================================================
   
   Critically, we only inherit from `UploadBatchFileCore` 
   (NOT from the extension trait). This ensures we do NOT
   pull in the generic method into the aggregator, keeping
   it object-safe.
*/
#[async_trait]
pub trait LanguageModelClientInterface<E: Debug>:
    RetrieveBatchById<Error = E>
    + GetBatchFileContent<Error = E>
    + UploadBatchFileCore<Error = E>
    + CreateBatch<Error = E>
    + WaitForBatchCompletion<Error = E>
    + Send
    + Sync
    + Debug
{
    // No generic methods here => object safe
}

/*
   Finally, the trait to fetch an LM client object as a trait object:
   This is now possible because we only reference the aggregator
   (which is object-safe).
*/
pub trait GetLanguageModelClient<E> {
    fn language_model_client(&self) -> Arc<dyn LanguageModelClientInterface<E>>;
}
