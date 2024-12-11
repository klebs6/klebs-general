crate::ix!();

#[derive(Debug, Serialize,Deserialize)]
pub struct Tweet {
    text:       String,
    created_at: String,
    id_str:     String,
    user:       TweetUser,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct TweetUser {
    screen_name: String,
}

#[derive(Debug, Serialize,Deserialize)]
pub struct Mention {
    created_at: String,
    id_str:     String,
    text:       String,
    user:       TweetUser,
}

pub struct TwitterAPI {
    client:       Client,
    signer:       OAuthSigner,
    bearer_token: String,
}

impl TwitterAPI {

    pub fn new(config: Config) -> Self {
        let signer = OAuthSigner::new(config.clone());
        TwitterAPI {
            client: Client::new(),
            signer,
            bearer_token: config.bearer_token().clone(),
        }
    }

    /// Retrieves the user's own tweets (home timeline)
    pub async fn get_user_tweets(&self, count: usize) -> Result<Vec<Tweet>, MessengerError> {
        let url = "https://api.twitter.com/1.1/statuses/user_timeline.json";

        let extended = "extended".to_string();

        let params = [
            ("count", &count.to_string()),
            ("tweet_mode", &extended), // To get full text
        ];

        let auth_header = self
            .signer
            .sign_request("GET", url, &params)?;

        let response = self
            .client
            .get(url)
            .header("Authorization", auth_header)
            .query(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let tweets: Vec<Tweet> = response.json().await?;
            Ok(tweets)
        } else {
            let error_text = response.text().await?;
            Err(MessengerError::HttpRequestError(
                reqwest::Error::new(
                    reqwest::StatusCode::from_u16(response.status().as_u16())
                        .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
                    format!("Failed to get user tweets: {}", error_text).into(),
                ),
            ))
        }
    }

    /// Retrieves mentions directed at the user within the past `interval` duration
    pub async fn get_mentions_within(
        &self,
        interval: Duration,
    ) -> Result<Vec<Mention>, MessengerError> {
        let url = "https://api.twitter.com/1.1/statuses/mentions_timeline.json";
        let params = [
            ("count", "200"), // Maximum allowed
            ("tweet_mode", "extended"),
        ];

        let auth_header = self
            .signer
            .sign_request("GET", url, &params)?;

        let response = self
            .client
            .get(url)
            .header("Authorization", auth_header)
            .query(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let mentions: Vec<Mention> = response.json().await?;
            let cutoff = Utc::now() - interval;
            let filtered_mentions: Vec<Mention> = mentions
                .into_iter()
                .filter(|m| {
                    if let Ok(dt) = DateTime::parse_from_rfc2822(&m.created_at) {
                        dt.with_timezone(&Utc) >= cutoff
                    } else {
                        false
                    }
                })
                .collect();

            // Sort mentions by creation time in descending order
            let mut sorted_mentions = filtered_mentions;
            sorted_mentions.sort_by_key(|m| Reverse(m.created_at.clone()));
            Ok(sorted_mentions)
        } else {
            let error_text = response.text().await?;
            Err(MessengerError::HttpRequestError(
                reqwest::Error::new(
                    reqwest::StatusCode::from_u16(response.status().as_u16())
                        .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
                    format!("Failed to get mentions: {}", error_text).into(),
                ),
            ))
        }
    }
}
