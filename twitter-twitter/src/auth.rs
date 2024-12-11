crate::ix!();

pub struct OAuthSigner {
    config: Config,
}

impl OAuthSigner {
    pub fn new(config: Config) -> Self {
        OAuthSigner { config }
    }

    pub fn sign_request(
        &self,
        method: &str,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<String, MessengerError> {
        let consumer = Token::new(
            self.config.api_key().clone(),
            self.config.api_key_secret().clone(),
        );
        
        let token = Token::new(
            self.config.access_token().clone(),
            self.config.access_token_secret().clone(),
        );
        
        let mut oauth_params = Params::new();
        oauth_params.add("oauth_signature_method", "HMAC-SHA1");
        oauth_params.add("oauth_consumer_key", &consumer.key);
        oauth_params.add("oauth_token", &token.key);
        oauth_params.add("oauth_nonce", &Nonce::new().to_string());
        oauth_params.add("oauth_timestamp", &Timestamp::now().to_string());
        oauth_params.add("oauth_version", "1.0");
        
        // Combine OAuth and request parameters
        let mut all_params = Params::new();
        for (k, v) in params {
            all_params.add(k, v);
        }
        for (k, v) in oauth_params.clone() {
            all_params.add(k, v);
        }
        
        // Create the signature base string
        let base_string = oauth::signature::base_string(method, url, &all_params);
        
        // Create the signing key
        let signing_key = format!(
            "{}&{}",
            oauth_percent_encode(&consumer.secret),
            oauth_percent_encode(&token.secret)
        );
        
        // Generate the signature
        let signature = HMAC_SHA1.sign(&base_string, &signing_key);
        
        // Add the signature to OAuth parameters
        oauth_params.add("oauth_signature", &signature);
        
        // Construct the Authorization header
        let auth_header = oauth_params.to_header();
        
        Ok(auth_header)
    }
}

