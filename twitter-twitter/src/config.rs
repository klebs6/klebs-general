crate::ix!();

#[derive(Clone,Getters,Setters,Builder,Debug)]
#[builder(setter(into, strip_option))]
#[getset(get = "pub", set = "pub")]
pub struct Config {
    api_key:             String,
    api_key_secret:      String,
    access_token:        String,
    access_token_secret: String,
    bearer_token:        String,
}

impl Config {

    pub fn from_files(base_path: &str) -> Result<Self, MessengerError> {
        let api_key             = read_secret_file(format!("{}/.twitter-api-key", base_path))?;
        let api_key_secret      = read_secret_file(format!("{}/.twitter-api-key-secret", base_path))?;
        let access_token        = read_secret_file(format!("{}/.twitter-access-token", base_path))?;
        let access_token_secret = read_secret_file(format!("{}/.twitter-access-token-secret", base_path))?;
        let bearer_token        = read_secret_file(format!("{}/.twitter-bearer-token", base_path))?;

        Ok(
            ConfigBuilder::default()
                .api_key(api_key)
                .api_key_secret(api_key_secret)
                .access_token(access_token)
                .access_token_secret(access_token_secret)
                .bearer_token(bearer_token)
                .build()?,
        )
    }
}

fn read_secret_file<P: AsRef<Path>>(path: P) -> Result<String, MessengerError> {
    Ok(fs::read_to_string(path).map(|s| s.trim().to_string())?)
}

