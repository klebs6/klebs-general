// ---------------- [ File: src/openai_client_handle.rs ]
crate::ix!();

pub trait OpenAIConfigInterface = async_openai::config::Config;

#[derive(Debug)]
pub struct OpenAIClientHandle<E> 
where
    E: Debug + Send + Sync + From<OpenAIClientError>,
{
    client: async_openai::Client<OpenAIConfig>,
    _marker: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E> LanguageModelClientInterface<E> for OpenAIClientHandle<E>
where
    // We unify each sub‐trait’s “type Error=E” with the needed bounds:
    E: From<OpenAIClientError>
     + From<std::io::Error>
     + Debug
     + Send
     + Sync,
{
    // No additional methods to define here, because it's just the aggregator.
    // The sub‐traits are already implemented above.
}

impl<E> OpenAIClientHandle<E> 
where
    E: Debug + Send + Sync + From<OpenAIClientError>, // so we can do `.map_err(E::from)?`
{
    pub fn new() -> Arc<Self> {

        info!("creating new OpenAI Client Handle");

        let openai_api_key 
            = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set");

        // Initialize OpenAI client with your API key
        let config = OpenAIConfig::new().with_api_key(openai_api_key);

        let client = async_openai::Client::with_config(config);

        Arc::new(Self { 
            client,
            _marker: std::marker::PhantomData::<E>,
        })
    }

    delegate!{
        to self.client {
            pub fn batches(&self) -> async_openai::Batches<OpenAIConfig>;
            pub fn files(&self) -> async_openai::Files<OpenAIConfig>;
        }
    }
}
